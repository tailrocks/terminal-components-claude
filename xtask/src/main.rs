//! Workspace checks (`COMPONENT_ARCHITECTURE.md` §16.5, §21 item 34, §22.7).
//!
//! ```text
//! cargo run -p xtask -- doc-check                # §3–§17 and §21-end references resolve
//! cargo run -p xtask -- boundary                 # every §16.5 / §22.7 grep and metadata check
//! cargo run -p xtask -- boundary --check <name>  # one named check
//! cargo run -p xtask -- bless-guard              # §16.3 / §36.5 baseline bless guard
//! cargo run -p xtask -- capture-matrix            # capture the app matrix into shots/
//! cargo run -p xtask -- list                     # commands and check names
//! ```
//!
//! Exit status is non-zero on any failure; `crates/tui/tests/architecture.rs`
//! runs the named checks through this binary.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use regex::Regex;
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

fn root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest.parent().map(Path::to_path_buf).unwrap_or(manifest)
}

// The capture contract is deliberately finite.  A reviewer can compare every
// generated cell, and a missing cell cannot hide behind an open-ended glob.
const CAPTURE_SIZES: [(u16, u16); 4] = [(80, 24), (100, 30), (120, 40), (160, 50)];
const CAPTURE_COLORS: [&str; 4] = ["truecolor", "256", "16", "mono"];
const CAPTURE_THEMES: [&str; 2] = ["junie", "paper"];
const CAPTURE_APPS: [CaptureApp; 3] = [
    CaptureApp {
        name: "showcase",
        binary: "showcase",
        extra_args: "",
    },
    CaptureApp {
        name: "tablepro",
        binary: "tablepro",
        extra_args: "",
    },
    CaptureApp {
        name: "jackin-preview",
        binary: "jackin-preview",
        // A paused frame makes the capture deterministic and avoids waiting
        // for the intro animation on every matrix cell.
        extra_args: "--motion paused --frame 0",
    },
];
const CAPTURE_MANIFEST: &str = "shots/capture-matrix.tsv";

// Independent acceptance pins for the visual review contract. Keep these
// literals separate from the generation axes: changing both declarations in
// one edit must not make a shortened matrix look complete.
const EXPECTED_CAPTURE_SIZES: [(u16, u16); 4] = [(80, 24), (100, 30), (120, 40), (160, 50)];
const EXPECTED_CAPTURE_COLORS: [&str; 4] = ["truecolor", "256", "16", "mono"];
const EXPECTED_CAPTURE_THEMES: [&str; 2] = ["junie", "paper"];
const EXPECTED_CAPTURE_APPS: [&str; 3] = ["showcase", "tablepro", "jackin-preview"];

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct CaptureApp {
    name: &'static str,
    binary: &'static str,
    extra_args: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct CaptureCase {
    app: CaptureApp,
    width: u16,
    height: u16,
    color: &'static str,
    theme: &'static str,
}

impl CaptureCase {
    fn size(self) -> String {
        format!("{}x{}", self.width, self.height)
    }

    fn shot_name(self) -> String {
        format!(
            "{}_{}_{}_{}",
            self.app.name,
            self.theme,
            self.color,
            self.size()
        )
    }
}

#[derive(Debug, Eq, PartialEq)]
struct CaptureRecord {
    case: CaptureCase,
    path: PathBuf,
    bytes: u64,
    sha256: String,
}

fn capture_matrix_cases() -> Vec<CaptureCase> {
    CAPTURE_APPS
        .into_iter()
        .flat_map(|app| {
            CAPTURE_SIZES.into_iter().flat_map(move |(width, height)| {
                CAPTURE_COLORS.into_iter().flat_map(move |color| {
                    CAPTURE_THEMES.into_iter().map(move |theme| CaptureCase {
                        app,
                        width,
                        height,
                        color,
                        theme,
                    })
                })
            })
        })
        .collect()
}

fn capture_matrix() -> Result<(), String> {
    let script = root().join("tools/capture.sh");
    if !script.is_file() {
        return Err(format!("capture script is missing: {}", rel(&script)));
    }

    let mut binaries = BTreeMap::new();
    for app in CAPTURE_APPS {
        let owners = binary_owners(app.binary)?;
        let binary = capture_binary_path(app.binary);
        validate_capture_binary(app.binary, &binary, &owners)?;
        binaries.insert(app.name, binary);
    }

    let shots = root().join("shots");
    fs::create_dir_all(&shots).map_err(|error| {
        format!(
            "cannot create capture output directory {}: {error}",
            rel(&shots)
        )
    })?;

    let cases = capture_matrix_cases();
    let mut records = Vec::with_capacity(cases.len());
    for (index, case) in cases.iter().copied().enumerate() {
        println!(
            "capture {}/{}: {} / {} / {} / {}",
            index.saturating_add(1),
            cases.len(),
            case.app.name,
            case.size(),
            case.theme,
            case.color
        );
        remove_capture_artifacts(case)?;

        let binary = binaries
            .get(case.app.name)
            .ok_or_else(|| format!("capture app is not declared: {}", case.app.name))?;

        let start_args = vec![
            "start".to_owned(),
            case.width.to_string(),
            case.height.to_string(),
        ];
        if let Err(error) = run_capture_script(&script, &start_args, case, binary) {
            let _ = stop_capture(&script, case, binary);
            return Err(error);
        }

        let shot_args = vec!["shot".to_owned(), case.shot_name()];
        let shot_result = run_capture_script(&script, &shot_args, case, binary);
        let stop_result = stop_capture(&script, case, binary);
        shot_result?;
        stop_result?;

        records.push(capture_record(case)?);
    }

    validate_capture_records(&cases, &records)?;
    let manifest = root().join(CAPTURE_MANIFEST);
    write_capture_manifest(&manifest, &records)?;
    println!(
        "capture-matrix: wrote {} ({} cells)",
        rel(&manifest),
        records.len()
    );
    Ok(())
}

fn binary_owners(binary: &str) -> Result<Vec<String>, String> {
    let md = metadata()?;
    let mut owners = Vec::new();
    for package in md.workspace_packages() {
        for target in &package.targets {
            if target.name == binary && target.kind.contains(&cargo_metadata::TargetKind::Bin) {
                owners.push(format!("{} ({})", package.name, target.src_path));
            }
        }
    }
    Ok(owners)
}

fn capture_binary_path(binary: &str) -> PathBuf {
    let variable = format!("BIN_{}", binary.replace('-', "_").to_ascii_uppercase());
    let configured = std::env::var_os(&variable).or_else(|| {
        if binary == "showcase" {
            std::env::var_os("BIN")
        } else {
            None
        }
    });
    match configured {
        Some(path) if !path.is_empty() => {
            let path = PathBuf::from(path);
            if path.is_absolute() {
                path
            } else {
                root().join(path)
            }
        }
        _ => {
            let target = std::env::var_os("CARGO_TARGET_DIR")
                .map(PathBuf::from)
                .map_or_else(
                    || root().join("target"),
                    |path| {
                        if path.is_absolute() {
                            path
                        } else {
                            root().join(path)
                        }
                    },
                );
            target.join("debug").join(binary)
        }
    }
}

fn validate_capture_binary(binary: &str, path: &Path, owners: &[String]) -> Result<(), String> {
    if owners.is_empty() {
        return Err(format!(
            "capture binary `{binary}` is missing from workspace metadata"
        ));
    }
    if owners.len() > 1 {
        return Err(format!(
            "duplicate capture binary `{binary}` targets: {}",
            owners.join(", ")
        ));
    }

    let metadata = fs::metadata(path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            format!("capture binary is missing: {}", path.display())
        } else {
            format!("cannot inspect capture binary {}: {error}", path.display())
        }
    })?;
    if !metadata.is_file() {
        return Err(format!(
            "capture binary is not a regular file: {}",
            path.display()
        ));
    }
    if metadata.len() == 0 {
        return Err(format!("capture binary is zero-size: {}", path.display()));
    }
    Ok(())
}

fn run_capture_script(
    script: &Path,
    args: &[String],
    case: CaptureCase,
    binary: &Path,
) -> Result<(), String> {
    let binary_arg = binary
        .strip_prefix(root())
        .unwrap_or(binary)
        .to_string_lossy()
        .into_owned();
    let output = Command::new("bash")
        .arg(script)
        .args(args)
        .current_dir(root())
        .env("BIN", binary_arg)
        .env("COLOR", case.color)
        .env("ARGS", capture_arguments(case))
        .output()
        .map_err(|error| {
            format!(
                "cannot run capture script for {}: {error}",
                case_label(case)
            )
        })?;
    if output.status.success() {
        return Ok(());
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    let detail = [stdout, stderr]
        .into_iter()
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join(" | ");
    if detail.is_empty() {
        Err(format!(
            "capture script failed for {} with status {}",
            case_label(case),
            output.status
        ))
    } else {
        Err(format!(
            "capture script failed for {} with status {}: {detail}",
            case_label(case),
            output.status
        ))
    }
}

fn capture_color_arg(color: &str) -> &str {
    if color == "mono" { "none" } else { color }
}

fn capture_arguments(case: CaptureCase) -> String {
    format!(
        "{} --theme {} --color {}",
        case.app.extra_args,
        case.theme,
        capture_color_arg(case.color)
    )
}

fn stop_capture(script: &Path, case: CaptureCase, binary: &Path) -> Result<(), String> {
    run_capture_script(script, &["stop".to_owned()], case, binary)
}

fn remove_capture_artifacts(case: CaptureCase) -> Result<(), String> {
    let base = root().join("shots").join(case.shot_name());
    for extension in ["ansi", "cursor", "html", "png", "txt"] {
        remove_file_if_present(&base.with_extension(extension))?;
    }
    Ok(())
}

fn remove_file_if_present(path: &Path) -> Result<(), String> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("cannot remove {}: {error}", path.display())),
    }
}

fn capture_record(case: CaptureCase) -> Result<CaptureRecord, String> {
    let path = root()
        .join("shots")
        .join(format!("{}.png", case.shot_name()));
    let (bytes, sha256) = file_digest(&path)?;
    Ok(CaptureRecord {
        case,
        path,
        bytes,
        sha256,
    })
}

fn file_digest(path: &Path) -> Result<(u64, String), String> {
    let metadata = fs::metadata(path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            format!("capture artifact is missing: {}", path.display())
        } else {
            format!(
                "cannot inspect capture artifact {}: {error}",
                path.display()
            )
        }
    })?;
    if !metadata.is_file() {
        return Err(format!(
            "capture artifact is not a regular file: {}",
            path.display()
        ));
    }
    if metadata.len() == 0 {
        return Err(format!("capture artifact is zero-size: {}", path.display()));
    }

    let mut file = File::open(path)
        .map_err(|error| format!("cannot read capture artifact {}: {error}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 16 * 1024];
    let mut bytes = 0_u64;
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| format!("cannot hash capture artifact {}: {error}", path.display()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        bytes = bytes.saturating_add(read as u64);
    }
    if bytes == 0 {
        return Err(format!("capture artifact is zero-size: {}", path.display()));
    }
    Ok((bytes, format!("{:x}", hasher.finalize())))
}

fn case_label(case: CaptureCase) -> String {
    format!(
        "{} / {} / {} / {}",
        case.app.name,
        case.size(),
        case.theme,
        case.color
    )
}

fn validate_capture_records(
    expected: &[CaptureCase],
    records: &[CaptureRecord],
) -> Result<(), String> {
    let expected: BTreeSet<CaptureCase> = expected.iter().copied().collect();
    let mut seen_cases = BTreeSet::new();
    let mut seen_paths = BTreeSet::new();
    let mut errors = Vec::new();

    for record in records {
        if !expected.contains(&record.case) {
            errors.push(format!(
                "unexpected capture cell: {}",
                case_label(record.case)
            ));
        }
        if !seen_cases.insert(record.case) {
            errors.push(format!(
                "duplicate capture cell: {}",
                case_label(record.case)
            ));
        }
        if !seen_paths.insert(record.path.clone()) {
            errors.push(format!(
                "duplicate capture artifact path: {}",
                rel(&record.path)
            ));
        }
        if record.bytes == 0 {
            errors.push(format!("zero-size capture artifact: {}", rel(&record.path)));
        }
        if record.sha256.len() != 64 || !record.sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
        {
            errors.push(format!("invalid sha256 for {}", rel(&record.path)));
        }
    }

    let missing: Vec<String> = expected
        .difference(&seen_cases)
        .map(|case| case_label(*case))
        .collect();
    if !missing.is_empty() {
        errors.push(format!("missing capture cell(s): {}", missing.join(", ")));
    }
    if records.len() != expected.len() {
        errors.push(format!(
            "capture matrix has {} record(s), expected {}",
            records.len(),
            expected.len()
        ));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

fn write_capture_manifest(path: &Path, records: &[CaptureRecord]) -> Result<(), String> {
    let mut manifest = String::from("path\tsize\tcolor\ttheme\tbytes\tsha256\n");
    for record in records {
        writeln!(
            manifest,
            "{}\t{}\t{}\t{}\t{}\t{}",
            rel(&record.path),
            record.case.size(),
            record.case.color,
            record.case.theme,
            record.bytes,
            record.sha256
        )
        .map_err(|error| format!("cannot format capture manifest: {error}"))?;
    }

    let temporary = path.with_extension("tsv.tmp");
    fs::write(&temporary, manifest)
        .map_err(|error| format!("cannot write {}: {error}", temporary.display()))?;
    if let Err(error) = fs::rename(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(format!("cannot finalize {}: {error}", path.display()));
    }
    Ok(())
}

/// §74.2 / §16.5. Keep the executable capture command tied to the complete
/// declared matrix. This is intentionally a cheap boundary check: the
/// capture itself is an explicit opt-in command, while this check makes a
/// missing app, axis, or script visible in `boundary` and `list`.
fn capture_script_contract_hits(script: &str) -> Vec<String> {
    [
        ("required BIN", ": \"${BIN:?"),
        ("COLOR default", "COLOR=${COLOR:-truecolor}"),
        ("ARGS expansion", "${ARGS:-}"),
        ("COLOR dispatch", "case \"$COLOR\" in"),
        ("truecolor branch", "truecolor)"),
        ("256-color branch", "256)"),
        ("16-color branch", "16)"),
        ("mono branch", "mono)"),
        ("terminal dimensions", "-x \"$cols\" -y \"$rows\""),
        ("tmux pane capture", "tmux capture-pane"),
        ("PNG conversion", "ansi2png.py"),
    ]
    .into_iter()
    .filter(|(_, fragment)| !script.contains(fragment))
    .map(|(label, fragment)| format!("capture script lacks {label}: `{fragment}`"))
    .chain((script.contains("BIN=${BIN:-target/debug/showcase}")).then(|| {
        "capture script still has the legacy showcase BIN default; the binary owner must be explicit"
            .to_owned()
    }))
    .collect()
}

fn capture_axes_contract_hits() -> Vec<String> {
    let mut hits = Vec::new();
    if CAPTURE_SIZES != EXPECTED_CAPTURE_SIZES {
        hits.push(format!(
            "capture sizes are {CAPTURE_SIZES:?}, expected {EXPECTED_CAPTURE_SIZES:?}"
        ));
    }
    if CAPTURE_COLORS != EXPECTED_CAPTURE_COLORS {
        hits.push(format!(
            "capture colors are {CAPTURE_COLORS:?}, expected {EXPECTED_CAPTURE_COLORS:?}"
        ));
    }
    if CAPTURE_THEMES != EXPECTED_CAPTURE_THEMES {
        hits.push(format!(
            "capture themes are {CAPTURE_THEMES:?}, expected {EXPECTED_CAPTURE_THEMES:?}"
        ));
    }
    let actual_apps: Vec<&str> = CAPTURE_APPS.iter().map(|app| app.name).collect();
    if actual_apps != EXPECTED_CAPTURE_APPS {
        hits.push(format!(
            "capture apps are {actual_apps:?}, expected {EXPECTED_CAPTURE_APPS:?}"
        ));
    }
    let expected_cells = 4usize * 4 * 2 * 3;
    let actual_cells = capture_matrix_cases().len();
    if actual_cells != expected_cells {
        hits.push(format!(
            "capture matrix declares {actual_cells} cells, expected {expected_cells}"
        ));
    }
    hits
}

fn capture_matrix_contract() -> Result<(), String> {
    let script = root().join("tools/capture.sh");
    if !script.is_file() {
        return Err(format!("capture script is missing: {}", rel(&script)));
    }

    let script_text = fs::read_to_string(&script)
        .map_err(|error| format!("cannot read capture script {}: {error}", rel(&script)))?;
    let mut errors = capture_axes_contract_hits();
    errors.extend(capture_script_contract_hits(&script_text));
    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }

    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("doc-check") => doc_check(),
        Some("boundary") => {
            let only = args
                .iter()
                .position(|a| a == "--check")
                .and_then(|i| args.get(i + 1).cloned());
            boundary(only.as_deref())
        }
        // §16.3 / §20.10 / §36.5. A named `boundary` check, so it inherits the
        // `ok`/`FAIL` formatting, the `N check(s) failed` tail and `xtask list`.
        Some("bless-guard") => boundary(Some("baseline_moves_are_classified")),
        Some("capture-matrix") => capture_matrix(),
        Some("list") => {
            println!("capture-matrix");
            for name in CHECKS.iter().map(|c| c.0) {
                println!("{name}");
            }
            Ok(())
        }
        _ => {
            eprintln!(
                "usage: xtask <doc-check | boundary [--check NAME] | bless-guard | capture-matrix | list>"
            );
            Err("no command".to_owned())
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("xtask: {e}");
            ExitCode::FAILURE
        }
    }
}

// ───────────────────────────── source scanning ─────────────────────────────

fn rust_files(dir: &Path) -> Vec<PathBuf> {
    if !dir.exists() {
        return Vec::new();
    }
    WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|x| x == "rs"))
        .map(|e| e.path().to_path_buf())
        .collect()
}

fn read(p: &Path) -> String {
    std::fs::read_to_string(p).unwrap_or_default()
}

fn rel(p: &Path) -> String {
    p.strip_prefix(root())
        .unwrap_or(p)
        .to_string_lossy()
        .replace('\\', "/")
}

/// Lines of a file with **each** `#[cfg(test)]` item removed.
///
/// The old version `break`ed at the first `#[cfg(test)]`, so a mid-file
/// `#[cfg(test)] pub(crate) const fn stats` left the whole rest of the file
/// unscanned by all 26 forbidden-pattern rules (MA-2). This skips exactly the
/// attributed item by brace matching and keeps scanning afterwards.
fn non_test_lines(text: &str) -> Vec<(usize, &str)> {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < lines.len() {
        let Some(line) = lines.get(i) else { break };
        if line.trim_start().starts_with("#[cfg(test)]") {
            i = skip_item(&lines, i.saturating_add(1));
            continue;
        }
        out.push((i.saturating_add(1), *line));
        i = i.saturating_add(1);
    }
    out
}

/// The index just past the item starting at `from`: any further attributes,
/// then either a braced body (matched, ignoring braces inside `"` strings and
/// `//` comments) or a single `;`-terminated line.
fn skip_item(lines: &[&str], from: usize) -> usize {
    let mut i = from;
    let mut depth = 0usize;
    let mut opened = false;
    while i < lines.len() {
        let Some(line) = lines.get(i) else { break };
        let code = code_line(line);
        let mut in_str = false;
        let mut prev_escape = false;
        for c in code.chars() {
            if in_str {
                if prev_escape {
                    prev_escape = false;
                } else if c == '\\' {
                    prev_escape = true;
                } else if c == '"' {
                    in_str = false;
                }
                continue;
            }
            match c {
                '"' => in_str = true,
                '{' => {
                    depth = depth.saturating_add(1);
                    opened = true;
                }
                '}' => depth = depth.saturating_sub(1),
                _ => {}
            }
        }
        i = i.saturating_add(1);
        if opened && depth == 0 {
            return i;
        }
        if !opened && code.trim_end().ends_with(';') {
            return i;
        }
    }
    i
}

fn code_line(line: &str) -> &str {
    // strip line comments so a documented rule can name a forbidden pattern
    let trimmed = line.trim_start();
    if trimmed.starts_with("//") {
        return "";
    }
    match line.find("//") {
        Some(i) if !line[..i].contains('"') => &line[..i],
        _ => line,
    }
}

// ───────────────────────────── boundary checks ─────────────────────────────

type Check = (&'static str, fn() -> Result<(), String>);

const CHECKS: &[Check] = &[
    (
        "no_deprecated_or_legacy_api_usage",
        no_deprecated_or_legacy_api_usage,
    ),
    (
        "dependency_graph_is_exactly_the_declared_set",
        dependency_graph_is_exactly_the_declared_set,
    ),
    (
        "library_has_no_application_dependency",
        library_has_no_application_dependency,
    ),
    (
        "no_domain_vocabulary_in_the_library",
        no_domain_vocabulary_in_the_library,
    ),
    (
        "palette_literals_are_confined_to_theme_builtins",
        palette_literals_are_confined_to_theme_builtins,
    ),
    ("no_raw_background_parameter", no_raw_background_parameter),
    ("no_public_geometry_or_cache", no_public_geometry_or_cache),
    (
        "no_fn_pointer_extension_points",
        no_fn_pointer_extension_points,
    ),
    ("no_todo_or_unimplemented", no_todo_or_unimplemented),
    ("no_unsafe", no_unsafe),
    (
        "no_static_bound_in_component_surface",
        no_static_bound_in_component_surface,
    ),
    ("draw_takes_shared_self", draw_takes_shared_self),
    (
        "closure_bearing_draw_signatures_are_exact",
        closure_bearing_draw_signatures_are_exact,
    ),
    (
        "grid_model_public_surface_is_exact",
        grid_model_public_surface_is_exact,
    ),
    (
        "field_kind_has_no_type_parameters",
        field_kind_has_no_type_parameters,
    ),
    ("cache_types_are_derived_only", cache_types_are_derived_only),
    (
        "capability_has_no_unicode_field",
        capability_has_no_unicode_field,
    ),
    (
        "no_boolean_capability_parameter_on_grid",
        no_boolean_capability_parameter_on_grid,
    ),
    ("core_is_backend_free", core_is_backend_free),
    (
        "msrv_and_edition_are_unchanged",
        msrv_and_edition_are_unchanged,
    ),
    ("no_unreachable_spin_loops", no_unreachable_spin_loops),
    (
        "ratatui_crossterm_is_named_in_exactly_two_files",
        ratatui_crossterm_is_named_in_exactly_two_files,
    ),
    ("every_named_test_exists", every_named_test_exists),
    (
        "conformance_covers_every_public_component",
        conformance_covers_every_public_component,
    ),
    (
        "legacy_forced_state_apis_are_absent",
        legacy_forced_state_apis_are_absent,
    ),
    (
        "examples_are_external_consumers",
        examples_are_external_consumers,
    ),
    (
        "reference_rendering_is_ui_scoped",
        reference_rendering_is_ui_scoped,
    ),
    ("binary_names_are_preserved", binary_names_are_preserved),
    ("capture_matrix_contract", capture_matrix_contract),
    (
        "app_libs_are_not_published_and_are_not_depended_on_by_the_library",
        app_libs_are_not_published_and_are_not_depended_on_by_the_library,
    ),
    (
        "applications_depend_only_on_the_library_facade",
        applications_depend_only_on_the_library_facade,
    ),
    (
        "no_generic_component_copies_in_applications",
        no_generic_component_copies_in_applications,
    ),
    (
        "no_owns_or_locate_in_applications",
        no_owns_or_locate_in_applications,
    ),
    (
        "baseline_moves_are_classified",
        baseline_moves_are_classified,
    ),
    ("props_are_built_once", props_are_built_once),
    (
        "showcase_covers_every_public_component",
        showcase_covers_every_public_component,
    ),
];

fn boundary(only: Option<&str>) -> Result<(), String> {
    let mut failures = Vec::new();
    for (name, f) in CHECKS {
        if only.is_some_and(|o| o != *name) {
            continue;
        }
        match f() {
            Ok(()) => println!("ok   {name}"),
            Err(e) => {
                println!("FAIL {name}\n{e}");
                failures.push(*name);
            }
        }
    }
    if only.is_some() && !CHECKS.iter().any(|c| Some(c.0) == only) {
        return Err(format!("unknown check {}", only.unwrap_or("")));
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{} check(s) failed: {}",
            failures.len(),
            failures.join(", ")
        ))
    }
}

/// A forbidden pattern: regex, allowed path substrings, reason.
struct Rule {
    n: u8,
    re: &'static str,
    allowed: &'static [&'static str],
    why: &'static str,
    /// Only lines that also match this (a `pub fn` signature, …).
    only_if: Option<&'static str>,
    /// Only under this path.
    under: Option<&'static str>,
}

const RULES: &[Rule] = &[
    Rule {
        n: 1,
        re: r"Buffer::get\b|Buffer::get_mut\b",
        allowed: &[],
        why: "deprecated",
        only_if: None,
        under: None,
    },
    Rule {
        n: 2,
        re: r"enable_raw_mode|disable_raw_mode",
        allowed: &["crates/tui/src/runtime/session.rs"],
        why: "R-18",
        only_if: None,
        under: None,
    },
    Rule {
        n: 3,
        re: r"EnterAlternateScreen|LeaveAlternateScreen|Enable(Mouse|Bracketed)|Disable(Mouse|Bracketed)",
        allowed: &["crates/tui/src/runtime/session.rs"],
        why: "R-18",
        only_if: None,
        under: None,
    },
    Rule {
        n: 4,
        re: r"\\x1b\[|\\u\{1b\}\[",
        allowed: &[],
        why: "raw ANSI",
        only_if: None,
        under: None,
    },
    Rule {
        n: 5,
        re: r"KeyboardEnhancementFlags",
        allowed: &[],
        why: "R-17",
        only_if: None,
        under: None,
    },
    Rule {
        n: 6,
        re: r"for\s+\w+\s+in\s+\w+\.top\(\)\.\.|\.left\(\)\.\.",
        allowed: &["crates/tui/src/ui/paint.rs"],
        why: "R-4",
        only_if: None,
        under: None,
    },
    Rule {
        n: 7,
        re: r"Rect::new\(",
        allowed: &[
            "crates/tui/src/layout.rs",
            "crates/tui/src/ui/",
            "crates/tui-testing/",
            "/tests/",
        ],
        why: "components receive rects",
        only_if: None,
        under: Some("crates/tui/src/components/"),
    },
    Rule {
        n: 8,
        re: r"Style::default\(\)",
        allowed: &[],
        why: "R-8, one spelling",
        only_if: None,
        under: None,
    },
    // NOTE (§4(j)-3): this rule cannot distinguish *construction*
    // (`st.fg = c` while building a `Style`) from *layering* (which R-9
    // forbids); the two occurrences in `ui/paint.rs` are construction and the
    // file is allow-listed by path for that reason.
    Rule {
        n: 9,
        re: r"Style::new\(\)\s*\.(fg|bg)\(|style\.(fg|bg)\(|\.add_modifier\(|\.remove_modifier\(|\.underline_color\(",
        allowed: &[
            "crates/tui/src/theme/",
            "crates/tui/src/ui/paint.rs",
            "crates/tui-testing/",
        ],
        why: "R-8",
        only_if: None,
        under: None,
    },
    Rule {
        n: 10,
        re: r"style::Stylize|\.(red|green|blue|yellow|magenta|cyan|white|black|gray|on_[a-z]+)\(\)",
        allowed: &["crates/tui-testing/"],
        why: "R-8",
        only_if: None,
        under: None,
    },
    Rule {
        n: 11,
        re: r"\bMasked\b",
        allowed: &[],
        why: "R-19",
        only_if: None,
        under: None,
    },
    Rule {
        n: 12,
        re: r"unicode_width::|UnicodeWidth(Str|Char)",
        allowed: &["crates/tui/src/text/measure.rs"],
        why: "R-1",
        only_if: None,
        under: None,
    },
    Rule {
        n: 13,
        re: r"unicode_segmentation::",
        allowed: &["crates/tui/src/text/"],
        why: "one segmentation site",
        only_if: None,
        under: None,
    },
    Rule {
        n: 14,
        re: r"\bratatui::|ratatui_widgets::|ratatui_macros::",
        allowed: &[],
        why: "R-20",
        only_if: None,
        under: None,
    },
    Rule {
        n: 15,
        re: r"\bScrollbar\b|ScrollbarState|ScrollbarOrientation|ScrollDirection",
        allowed: &[],
        why: "§22.2 item 13",
        only_if: None,
        under: None,
    },
    Rule {
        n: 16,
        re: r"\b(Block|Paragraph|Padding|BorderType|Borders|Shadow|Clear)::|widgets::Clear\b|Fill::new|\bDimmed\b",
        allowed: &[],
        why: "R-20",
        only_if: None,
        under: None,
    },
    Rule {
        n: 17,
        re: r"#\[allow\(",
        allowed: &["crates/tui-testing/"],
        why: "#[expect(…, reason)] only",
        only_if: None,
        under: None,
    },
    Rule {
        n: 18,
        re: r"LazyLock|OnceLock|static mut|thread_local!",
        allowed: &["crates/tui-testing/", "xtask/"],
        why: "no process-global state",
        only_if: None,
        under: None,
    },
    Rule {
        n: 19,
        re: r"\.unwrap\(\)|\.expect\(|panic!|todo!|unimplemented!",
        // `ui/derived.rs` is the single documented exception: `dyn
        // Any::downcast_mut` returns `Option` and safe Rust cannot express
        // "keyed by TypeId::of::<T>() ⇒ holds Box<T>". A named path shows the
        // exception; the alternative was a livelock (BL-2).
        allowed: &[
            "crates/tui-testing/",
            "xtask/",
            "crates/tui/src/ui/derived.rs",
        ],
        why: "goal §10",
        only_if: None,
        under: None,
    },
    Rule {
        n: 20,
        re: r"fn render\(&mut self|fn draw\(&mut self",
        allowed: &[],
        why: "G2",
        only_if: None,
        under: Some("crates/tui/src/components/"),
    },
    Rule {
        n: 21,
        re: r"\bbg:\s*(ratatui_core::style::)?Color\b",
        allowed: &[],
        why: "§16.5 no raw background parameter",
        only_if: Some(r"\bpub (const )?fn\b"),
        under: None,
    },
    Rule {
        n: 22,
        // §22.7's broad regex: a narrowed one lets `Color::Rgb(r, g, b)` from
        // computed values through anywhere (D-10). The exceptions are named
        // *paths*, which are printed and reviewable; path exceptions do not
        // feed the "legacy_api.txt must be empty" condition.
        re: r"Color::Rgb\(|Color::from_u32\(|#[0-9a-fA-F]{6}\b",
        allowed: &[
            "crates/tui/src/theme/builtin/junie.rs",
            "crates/tui/src/theme/builtin/paper.rs",
            // derives a colour arithmetically from the seeds (L* ladders)
            "crates/tui/src/theme/builder.rs",
            // reconstructs `(r, g, b)` when downgrading to a palette
            "crates/tui/src/theme/downgrade.rs",
            "/tests/fixtures/",
            "crates/tui-testing/",
        ],
        why: "R-10 colour literals only in builtins",
        only_if: None,
        under: None,
    },
    Rule {
        n: 23,
        re: r"set_cursor_position",
        allowed: &["crates/tui/src/runtime.rs", "crates/tui-testing/"],
        why: "R-7",
        only_if: None,
        under: None,
    },
    Rule {
        n: 24,
        re: r"Layout::|Constraint::|Flex::|Spacing::",
        allowed: &[],
        why: "R-13",
        only_if: None,
        under: Some("crates/tui/src/components/"),
    },
    Rule {
        n: 25,
        re: r"\.child\(|\.owns\(|\.locate\b|scrollbar::id_for|WidgetId",
        allowed: &[],
        why: "§16.5 dispatch",
        only_if: None,
        under: None,
    },
    Rule {
        n: 26,
        re: r"SmallVec|smallvec::",
        allowed: &[],
        why: "§22.4",
        only_if: None,
        under: None,
    },
    Rule {
        n: 27,
        re: r"CrosstermBackend|ratatui_crossterm::crossterm::(?:terminal|execute|cursor|style)",
        allowed: &["crates/tui/src/runtime/session.rs"],
        why: "§22.1: the backend lives in runtime/session.rs only",
        only_if: None,
        under: None,
    },
    Rule {
        n: 28,
        re: r"spin_loop",
        allowed: &[],
        why: "BL-2: a livelock is strictly worse than a panic",
        only_if: None,
        under: None,
    },
];

/// Return a diagnostic when a required source root cannot provide any input.
///
/// A missing or empty root is not equivalent to a clean scan: a moved package
/// can otherwise make every forbidden-pattern rule green by removing the files
/// it was meant to inspect (§47.5).
fn scan_root_error(path: &Path, is_dir: bool, rust_file_count: usize) -> Option<String> {
    if !is_dir {
        return Some(format!(
            "scan root {} is missing or not a directory",
            rel(path)
        ));
    }
    if rust_file_count == 0 {
        return Some(format!(
            "scan root {} contains no Rust files; an empty scan is not a pass",
            rel(path)
        ));
    }
    None
}

fn checked_scan_root(path: &Path) -> Result<Vec<PathBuf>, String> {
    let files = rust_files(path);
    if let Some(error) = scan_root_error(path, path.is_dir(), files.len()) {
        Err(error)
    } else {
        Ok(files)
    }
}

/// Source roots for the forbidden-pattern scan.
///
/// The two library roots are always due. `apps/*/src` roots become due when an
/// `apps` tree is present; an empty `apps` directory or an application without
/// a Rust source file fails closed instead of making the scan green-empty.
fn scan_roots() -> Result<Vec<PathBuf>, String> {
    let r = root();
    let mut v = vec![r.join("crates/tui/src"), r.join("crates/tui-testing/src")];
    let apps = r.join("apps");
    if apps.exists() {
        if !apps.is_dir() {
            return Err(format!(
                "scan root {} is missing or not a directory",
                rel(&apps)
            ));
        }
        let mut app_count = 0usize;
        for entry in std::fs::read_dir(&apps)
            .map_err(|e| format!("cannot enumerate scan root {}: {e}", rel(&apps)))?
        {
            let entry = entry.map_err(|e| format!("cannot read {} entry: {e}", rel(&apps)))?;
            if !entry.path().is_dir() {
                continue;
            }
            app_count = app_count.saturating_add(1);
            v.push(entry.path().join("src"));
        }
        if app_count == 0 {
            return Err(format!(
                "scan root {} contains no application roots; an empty scan is not a pass",
                rel(&apps)
            ));
        }
    }
    for path in &v {
        checked_scan_root(path)?;
    }
    Ok(v)
}

fn read_allow(name: &str) -> BTreeMap<String, String> {
    let p = root().join("crates/tui/tests/allow").join(name);
    let mut out = BTreeMap::new();
    for line in read(&p).lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (entry, why) = line.split_once("  #").unwrap_or((line, ""));
        out.insert(entry.trim().to_owned(), why.trim().to_owned());
    }
    out
}

fn no_deprecated_or_legacy_api_usage() -> Result<(), String> {
    let allow = read_allow("legacy_api.txt");
    for (entry, why) in &allow {
        println!("legacy_api allow-list: {entry}  # {why}");
    }
    let mut hits = Vec::new();
    let roots = scan_roots()?;
    for rule in RULES {
        let re = Regex::new(rule.re).map_err(|e| e.to_string())?;
        let only_if = rule
            .only_if
            .map(|r| Regex::new(r).map_err(|e| e.to_string()))
            .transpose()?;
        for dir in &roots {
            for file in rust_files(dir) {
                let path = rel(&file);
                if rule.allowed.iter().any(|a| path.contains(a)) {
                    continue;
                }
                if let Some(u) = rule.under
                    && !path.contains(u)
                {
                    continue;
                }
                let text = read(&file);
                for (ln, line) in non_test_lines(&text) {
                    let code = code_line(line);
                    if !re.is_match(code) {
                        continue;
                    }
                    if only_if.as_ref().is_some_and(|o| !o.is_match(code)) {
                        continue;
                    }
                    let key = format!("{path}:{ln}");
                    if allow.contains_key(&key) {
                        continue;
                    }
                    hits.push(format!(
                        "rule {} ({}): {key}: {}",
                        rule.n,
                        rule.why,
                        line.trim()
                    ));
                }
            }
        }
    }
    if !allow.is_empty() {
        hits.push(format!(
            "the allow-list must be empty; it has {} entries",
            allow.len()
        ));
    }
    if hits.is_empty() {
        Ok(())
    } else {
        Err(hits.join("\n"))
    }
}

// ─────────────────────── the named-test inventory (§21 item 28) ───────────────────────

/// The section of `COMPONENT_ARCHITECTURE.md` between two headings.
fn doc_section(doc: &str, from: &str, to: &str) -> String {
    let Some(a) = doc.find(from) else {
        return String::new();
    };
    let rest = doc.get(a..).unwrap_or_default();
    match rest.find(to) {
        Some(b) => rest.get(..b).unwrap_or_default().to_owned(),
        None => rest.to_owned(),
    }
}

/// Backticked snake-case identifiers at parenthesis depth 0, with HTML
/// comments removed. The listed names are at depth 0; the prose that explains
/// them is inside `(…)`, which is exactly how §16.1 is written.
fn doc_test_names(text: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut depth = 0i32;
    let bytes = text.as_bytes();
    let mut i = 0usize;
    while i < text.len() {
        let rest = text.get(i..).unwrap_or_default();
        if rest.starts_with("<!--") {
            i = rest.find("-->").map_or(text.len(), |j| i + j + 3);
            continue;
        }
        let c = bytes.get(i).copied().unwrap_or(b' ');
        match c {
            b'(' => depth = depth.saturating_add(1),
            b')' => depth = depth.saturating_sub(1),
            b'`' => {
                let body = rest.get(1..).unwrap_or_default();
                let Some(end) = body.find('`') else {
                    break;
                };
                let token = body.get(..end).unwrap_or_default();
                if depth == 0 {
                    let last = token.rsplit("::").next().unwrap_or(token);
                    let snake = last
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
                    if snake
                        && last.matches('_').count() >= 2
                        && last.starts_with(|c: char| c.is_ascii_lowercase())
                    {
                        out.insert(last.to_owned());
                    }
                }
                i = i + 1 + end + 1;
                continue;
            }
            _ => {}
        }
        i = i.saturating_add(
            text.get(i..)
                .and_then(|r| r.chars().next())
                .map_or(1, char::len_utf8),
        );
    }
    out
}

/// The first cell of every `| \`name\` | … |` row.
fn doc_table_names(text: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for line in text.lines() {
        let line = line.trim();
        if !line.starts_with('|') {
            continue;
        }
        let cell = line.trim_start_matches('|').trim();
        let Some(rest) = cell.strip_prefix('`') else {
            continue;
        };
        let Some(end) = rest.find('`') else { continue };
        let name = rest.get(..end).unwrap_or_default();
        let last = name.rsplit("::").next().unwrap_or(name);
        if last
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
            && last.matches('_').count() >= 2
        {
            out.insert(last.to_owned());
        }
    }
    out
}

/// Every `#[test]`-annotated function name in the workspace's sources.
///
/// §21 item 28 words this as `cargo test --workspace -- --list`. This scans
/// the sources for the same thing — every `#[test] fn name` — because the
/// check runs *inside* `cargo test --test architecture` and a nested
/// `cargo test --workspace --test perf --test perf_collections --release -- --list` would rebuild the
/// world in a second profile on every architecture run. The two enumerate the
/// same set; source scanning additionally sees `cfg`-gated tests, which for a
/// one-directional "the name exists" check is the safer direction.
fn declared_test_names() -> BTreeSet<String> {
    let r = root();
    let mut dirs = vec![
        r.join("crates"),
        r.join("src"),
        r.join("tests"),
        r.join("xtask/src"),
    ];
    if r.join("apps").exists() {
        dirs.push(r.join("apps"));
    }
    let Ok(re) = Regex::new(r"\bfn\s+([a-z_][a-z0-9_]*)\s*\(") else {
        return BTreeSet::new();
    };
    let mut out = BTreeSet::new();
    for dir in dirs {
        for file in rust_files(&dir) {
            if rel(&file).contains("/target/") {
                continue;
            }
            let text = read(&file);
            let lines: Vec<&str> = text.lines().collect();
            for (i, line) in lines.iter().enumerate() {
                if !line.trim_start().starts_with("#[test]") {
                    continue;
                }
                for l in lines.iter().skip(i.saturating_add(1)).take(4) {
                    if let Some(c) = re.captures(l) {
                        if let Some(m) = c.get(1) {
                            out.insert(m.as_str().to_owned());
                        }
                        break;
                    }
                }
            }
        }
    }
    // `trybuild` cases are file names, not `#[test]` functions
    if let Ok(entries) = std::fs::read_dir(r.join("crates/tui/tests/ui")) {
        for e in entries.filter_map(Result::ok) {
            if let Some(stem) = e.path().file_stem().and_then(|s| s.to_str()) {
                out.insert(stem.to_owned());
            }
        }
    }
    out
}

/// The names deferred to a later slice, with the slice that owns each.
fn named_tests_allow() -> BTreeMap<String, String> {
    let p = root().join("xtask/named_tests_allow.txt");
    let mut out = BTreeMap::new();
    for line in read(&p).lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (name, why) = line.split_once("  #").unwrap_or((line, ""));
        out.insert(name.trim().to_owned(), why.trim().to_owned());
    }
    out
}

/// Benchmarks §16.6 records as **deleted**: their deletion is asserted by
/// line-absence in `perf_baseline.txt` (§21 item 28), so the name must not
/// appear as the first field of any data row of any baseline in the tree.
const DELETED_PERF_ROWS: [(&str, &str); 1] = [(
    "capsule_pane_clone_4x2000",
    "§21 item 10 / §16.6: `Capsule` clones no viewport per frame, so the \
     benchmark is deleted (Slice 7, apps/jackin-preview)",
)];

/// Every `perf_baseline.txt` in the tree, build output excluded.
///
/// There are two, holding disjoint rows: the root `tests/perf_baseline.txt` —
/// the WP-0 pre-refactor baseline tagged `perf/baseline`, still carrying the
/// application benchmarks that Slices 5–7 move out — and
/// `crates/tui/tests/perf_baseline.txt`, the new library baseline written by
/// `crates/tui-testing`. A deletion assertion aimed at one file alone passes by
/// construction when the row lives in the other, and stops meaning anything
/// the moment a slice moves a benchmark; scanning all of them is invariant
/// under those moves.
fn perf_baselines() -> Vec<PathBuf> {
    WalkDir::new(root())
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !(e.file_type().is_dir() && (name == "target" || name == ".git"))
        })
        .filter_map(Result::ok)
        .filter(|e| e.file_name() == "perf_baseline.txt")
        .map(|e| e.path().to_path_buf())
        .collect()
}

/// The §21 item 28 deletion assertion: one message per `DELETED_PERF_ROWS`
/// name still present in a baseline, naming the file and line so the failure
/// is actionable rather than a bare "must be ABSENT".
fn surviving_deleted_perf_rows() -> Vec<String> {
    let mut hits = Vec::new();
    for path in perf_baselines() {
        for (i, line) in read(&path).lines().enumerate() {
            let row = line.trim();
            if row.is_empty() || row.starts_with('#') {
                continue;
            }
            let name = row.split_whitespace().next().unwrap_or_default();
            for (deleted, why) in DELETED_PERF_ROWS {
                if name == deleted {
                    hits.push(format!(
                        "{}:{}: {deleted} must be ABSENT from perf_baseline.txt — {why}",
                        rel(&path),
                        i.saturating_add(1)
                    ));
                }
            }
        }
    }
    hits
}

fn every_named_test_exists() -> Result<(), String> {
    let doc = read(&root().join("COMPONENT_ARCHITECTURE.md"));
    if doc.is_empty() {
        return Err("COMPONENT_ARCHITECTURE.md not found".to_owned());
    }
    let mut want = BTreeSet::new();
    want.extend(doc_test_names(&doc_section(&doc, "### 16.1", "### 16.2")));
    // §16.2: the suite-level bullets only (the per-case names are generated)
    for line in doc_section(&doc, "Suite-level tests (emitted once", "---").lines() {
        if line.trim_start().starts_with("* `") {
            want.extend(doc_test_names(line));
        }
    }
    // §16.4: only the "New application coverage" list names tests; the rest
    // of the section is the `Harness` API contract
    want.extend(doc_test_names(&doc_section(
        &doc,
        "**New application coverage**",
        "### 16.5",
    )));
    // §16.5: the architecture-check table. Its absence was a blind spot
    // covering EVERY boundary check (§47.4): all nine names that follow are
    // listed only here and in Appendix B, so until this line existed their
    // absence could not fail any gate — which is how three missing `apps/`
    // guards came to be found by a person reading the document.
    want.extend(doc_table_names(&doc_section(&doc, "### 16.5", "### 16.6")));
    want.extend(doc_table_names(&doc_section(&doc, "### 16.6", "## 17.")));

    let have = declared_test_names();
    let allow = named_tests_allow();
    let mut missing = Vec::new();
    let mut deferred = 0usize;
    for name in &want {
        if have.contains(name) {
            continue;
        }
        if allow.contains_key(name) {
            deferred = deferred.saturating_add(1);
            continue;
        }
        missing.push(name.clone());
    }
    println!(
        "every_named_test_exists: {} names in §16.1/§16.2/§16.4/§16.5/§16.6, {} present, {deferred} deferred (xtask/named_tests_allow.txt)",
        want.len(),
        want.len()
            .saturating_sub(missing.len())
            .saturating_sub(deferred)
    );
    // the §21 item 28 deletion assertion
    missing.extend(surviving_deleted_perf_rows());
    // an allow-list entry that is now satisfied must be removed
    let stale: Vec<&String> = allow.keys().filter(|n| have.contains(*n)).collect();
    if !stale.is_empty() {
        missing.push(format!("stale entries in named_tests_allow.txt: {stale:?}"));
    }
    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "named tests missing (add the test, or defer it in xtask/named_tests_allow.txt with the owning slice):\n  {}",
            missing.join("\n  ")
        ))
    }
}

/// One `name => NameCase` entry of a `conformance_suite!` invocation.
struct SuiteEntry {
    module: syn::Ident,
    case: syn::Type,
}

impl syn::parse::Parse for SuiteEntry {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        let module: syn::Ident = input.parse()?;
        input.parse::<syn::Token![=>]>()?;
        let case: syn::Type = input.parse()?;
        Ok(Self { module, case })
    }
}

/// The case type names **registered** in `conformance.rs`'s
/// `conformance_suite!` invocation, read from the macro's token stream.
///
/// Never a substring search over the file text. Incident 2: `select =>
/// SelectCase,` is commented out of the list while the identifier
/// `SelectCase` occurs nine more times in the same file — in prose, in
/// `struct SelectCase;`, in its `impl Conformance` block and in three
/// standalone tests — so `text.contains("SelectCase")` reported a withheld
/// case as registered and the §16.2 coverage gate could not fail. Comments
/// are not tokens, so nothing outside the list is visible here.
fn registered_conformance_cases(path: &Path) -> Result<BTreeSet<String>, String> {
    let text = read(path);
    if text.is_empty() {
        return Err(format!("{} not found", rel(path)));
    }
    let ast = syn::parse_file(&text).map_err(|e| format!("{} does not parse: {e}", rel(path)))?;
    let mut cases = BTreeSet::new();
    let mut invocations = 0usize;
    collect_suite_entries(&ast.items, &mut cases, &mut invocations)?;
    if invocations == 0 {
        return Err(format!(
            "{}: no conformance_suite! invocation — the §16.2 coverage gate has nothing to read",
            rel(path)
        ));
    }
    Ok(cases)
}

/// Walks items (and inline modules) for `conformance_suite!` invocations and
/// parses each body as `name => NameCase,` entries.
fn collect_suite_entries(
    items: &[syn::Item],
    cases: &mut BTreeSet<String>,
    invocations: &mut usize,
) -> Result<(), String> {
    for item in items {
        match item {
            syn::Item::Mod(m) => {
                if let Some((_, inner)) = &m.content {
                    collect_suite_entries(inner, cases, invocations)?;
                }
            }
            syn::Item::Macro(m)
                if m.mac
                    .path
                    .segments
                    .last()
                    .is_some_and(|s| s.ident == "conformance_suite") =>
            {
                *invocations = invocations.saturating_add(1);
                let entries = m
                    .mac
                    .parse_body_with(
                        syn::punctuated::Punctuated::<SuiteEntry, syn::Token![,]>::parse_terminated,
                    )
                    .map_err(|e| format!("conformance_suite! body does not parse: {e}"))?;
                for entry in entries {
                    let syn::Type::Path(tp) = &entry.case else {
                        return Err(format!(
                            "conformance_suite!: entry `{}` registers a non-path case type",
                            entry.module
                        ));
                    };
                    let seg = tp.path.segments.last().ok_or_else(|| {
                        format!(
                            "conformance_suite!: entry `{}` registers an empty path",
                            entry.module
                        )
                    })?;
                    cases.insert(seg.ident.to_string());
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// §16.2: `conformance.rs`'s `conformance_suite!` must list a case for every
/// public component, so adding a component without registering it fails CI.
fn conformance_covers_every_public_component() -> Result<(), String> {
    let dir = root().join("crates/tui/src/components");
    let registered = registered_conformance_cases(&root().join("crates/tui/tests/conformance.rs"))?;
    let mut missing = Vec::new();
    let mut covered = 0usize;
    for (path, ast) in parse_files(&dir) {
        if path.ends_with("/mod.rs") {
            continue;
        }
        // the component type is the public struct whose inherent impl has
        // `pub fn draw(&self, ui: &mut Ui<'_>, …)`
        let mut drawn: BTreeSet<String> = BTreeSet::new();
        for item in &ast.items {
            if let syn::Item::Impl(im) = item
                && im.trait_.is_none()
                && let syn::Type::Path(tp) = im.self_ty.as_ref()
                && let Some(seg) = tp.path.segments.last()
            {
                for it in &im.items {
                    if let syn::ImplItem::Fn(f) = it
                        && f.sig.ident == "draw"
                        && matches!(f.vis, syn::Visibility::Public(_))
                    {
                        drawn.insert(seg.ident.to_string());
                    }
                }
            }
        }
        let public: BTreeSet<String> = ast
            .items
            .iter()
            .filter_map(|i| match i {
                syn::Item::Struct(s) if matches!(s.vis, syn::Visibility::Public(_)) => {
                    Some(s.ident.to_string())
                }
                _ => None,
            })
            .collect();
        for name in drawn.intersection(&public) {
            let case = format!("{name}Case");
            if registered.contains(&case) {
                covered = covered.saturating_add(1);
            } else {
                missing.push(format!(
                    "{path}: {name} is not certified — the conformance_suite! list in \
                     crates/tui/tests/conformance.rs has no `=> {case}` entry \
                     (mentioning {case} elsewhere in that file does not register it)"
                ));
            }
        }
    }
    println!(
        "conformance_covers_every_public_component: {covered} component(s) registered, \
         {} entr(y/ies) in conformance_suite!",
        registered.len()
    );
    if missing.is_empty() {
        Ok(())
    } else {
        Err(missing.join("\n"))
    }
}

const LEGACY_FORCED_STATE_APIS: [&str; 2] = ["state_override", "inherit_forced"];

fn legacy_forced_state_hits(path: &str, source: &str) -> Vec<String> {
    let mut hits = Vec::new();
    for (line_number, line) in non_test_lines(source) {
        let code = code_line(line);
        for legacy in LEGACY_FORCED_STATE_APIS {
            if code
                .split(|character: char| !character.is_alphanumeric() && character != '_')
                .any(|identifier| identifier == legacy)
            {
                hits.push(format!("{path}:{line_number}: {}", line.trim()));
            }
        }
    }
    hits
}

/// Option-B A11 boundary: retired builder and propagation hooks cannot survive
/// in shipped library code, privately or publicly.
fn legacy_forced_state_apis_are_absent() -> Result<(), String> {
    let mut hits = Vec::new();
    for dir in [root().join("crates"), root().join("apps")] {
        for file in rust_files(&dir) {
            hits.extend(legacy_forced_state_hits(&rel(&file), &read(&file)));
        }
    }
    if hits.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "legacy forced-state APIs must be absent from production and the public surface:\n{}",
            hits.join("\n")
        ))
    }
}

fn reference_path_is_fixture(path: &str) -> bool {
    path.starts_with("apps/")
        || path.contains("/examples/")
        || path.contains("/tests/")
        || path.starts_with("crates/tui-testing/")
}

fn ui_reference_hits(path: &str, source: &str) -> Vec<String> {
    if reference_path_is_fixture(path) {
        return Vec::new();
    }
    let call = Regex::new(r"(?:\bUi\s*::\s*reference|\.\s*reference)\s*\(")
        .expect("valid Ui::reference call regex");
    non_test_lines(source)
        .into_iter()
        .filter(|(_, line)| call.is_match(code_line(line)))
        .map(|(line_number, line)| format!("{path}:{line_number}: {}", line.trim()))
        .collect()
}

/// Option-B A11 boundary: reference rendering belongs to fixture/application
/// orchestration. Shipped component and runtime code may never invoke it.
fn reference_rendering_is_ui_scoped() -> Result<(), String> {
    let mut hits = Vec::new();
    for dir in [root().join("crates"), root().join("apps")] {
        for file in rust_files(&dir) {
            let path = rel(&file);
            hits.extend(ui_reference_hits(&path, &read(&file)));
        }
    }
    if hits.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "`Ui::reference` calls are restricted to apps, examples, tests, testing support, \
             and `#[cfg(test)]` items; production calls are forbidden:\n{}",
            hits.join("\n")
        ))
    }
}

/// §16.5: the examples are external consumers — they compile against the
/// public facade only, with no `#[path]`, no `include!` and no private paths.
fn examples_are_external_consumers() -> Result<(), String> {
    let dir = root().join("crates/tui/examples");
    let mut hits = Vec::new();
    let mut n = 0usize;
    for file in rust_files(&dir) {
        n = n.saturating_add(1);
        let path = rel(&file);
        let text = read(&file);
        for (i, line) in text.lines().enumerate() {
            let code = code_line(line);
            if code.contains("#[path") || code.contains("include!") {
                hits.push(format!("{path}:{}: {}", i.saturating_add(1), line.trim()));
            }
        }
    }
    if n == 0 {
        return Err("no examples found under crates/tui/examples".to_owned());
    }
    let status = Command::new("cargo")
        .args(["build", "-p", LIB, "--examples", "-q"])
        .current_dir(root())
        .status()
        .map_err(|e| e.to_string())?;
    if !status.success() {
        hits.push("cargo build -p junie-tui --examples failed".to_owned());
    }
    println!("examples_are_external_consumers: {n} example(s)");
    if hits.is_empty() {
        Ok(())
    } else {
        Err(hits.join("\n"))
    }
}

/// §13 / §16.5 / §73. Where a configured construction may be built.
///
/// `apps/**/src` is the Slice-5 screen scope the rule was written for;
/// `crates/tui/examples` is the same shape written by an external consumer;
/// `crates/tui/src/components` is the composite-component scope §73 added,
/// because a `Form`, a `Dialog` or a `Wizard` builds child components across
/// both phases exactly like a screen does, and until §73 nothing looked at
/// it. A root that does not exist yet contributes nothing; a root that exists
/// and holds no Rust file is a failure, so this cannot go quiet by a move.
const PROPS_ROOTS: [&str; 3] = ["apps", "crates/tui/examples", "crates/tui/src/components"];

/// A chain method that ends configuration because it is a *phase* call: the
/// receiver stops being the component from there on.
fn is_phase_method(name: &str) -> bool {
    matches!(name, "update" | "draw" | "measure" | "layer" | "erase")
        || name.starts_with("update_")
        || name.starts_with("draw_")
}

/// Whether `ident` is a `const` name — the `const Id` the rule keys on.
/// A lower-case path is a local, a field or a call, and a dynamic id is not
/// keyed by this rule at all.
fn is_const_name(ident: &str) -> bool {
    ident.len() > 1
        && ident
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
        && ident.starts_with(|c: char| c.is_ascii_uppercase())
}

fn path_text(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

/// `Type::new(CONST_ID, …)` as `Type::new(CONST_ID)`, or `None` when the
/// expression is not a component construction keyed by a `const Id`.
///
/// The const is the **first** const-named argument, not strictly the first
/// argument: `FieldSpec::new(NAME, …)` leads with its id and
/// `Field::new("Name", …)` leads with its label.
fn ctor_const_key(expr: &syn::Expr) -> Option<String> {
    let syn::Expr::Call(call) = expr else {
        return None;
    };
    let syn::Expr::Path(func) = call.func.as_ref() else {
        return None;
    };
    let mut segments = func.path.segments.iter().rev();
    let last = segments.next()?;
    if last.ident != "new" {
        return None;
    }
    let ty = segments.next()?;
    let name = ty.ident.to_string();
    if !name.starts_with(|c: char| c.is_ascii_uppercase()) {
        return None;
    }
    let id = call.args.iter().find_map(|a| match a {
        syn::Expr::Path(p) => {
            let text = path_text(&p.path);
            let last = p.path.segments.last()?.ident.to_string();
            is_const_name(&last).then_some(text)
        }
        _ => None,
    })?;
    Some(format!("{name}::new({id})"))
}

fn is_cfg_test(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| {
        a.path().is_ident("cfg")
            && match &a.meta {
                syn::Meta::List(l) => l.tokens.to_string().contains("test"),
                syn::Meta::Path(_) | syn::Meta::NameValue(_) => false,
            }
    })
}

fn item_attrs(item: &syn::Item) -> &[syn::Attribute] {
    match item {
        syn::Item::Const(i) => &i.attrs,
        syn::Item::Enum(i) => &i.attrs,
        syn::Item::ExternCrate(i) => &i.attrs,
        syn::Item::Fn(i) => &i.attrs,
        syn::Item::ForeignMod(i) => &i.attrs,
        syn::Item::Impl(i) => &i.attrs,
        syn::Item::Macro(i) => &i.attrs,
        syn::Item::Mod(i) => &i.attrs,
        syn::Item::Static(i) => &i.attrs,
        syn::Item::Struct(i) => &i.attrs,
        syn::Item::Trait(i) => &i.attrs,
        syn::Item::TraitAlias(i) => &i.attrs,
        syn::Item::Type(i) => &i.attrs,
        syn::Item::Union(i) => &i.attrs,
        syn::Item::Use(i) => &i.attrs,
        _ => &[],
    }
}

/// A function discovered while scanning one source file.
struct PropsFunction {
    module: String,
    name: String,
    private: bool,
    calls: Vec<String>,
}

/// One direct component construction. `configured` means a builder method
/// occurs before the phase call (or the construction is a props-returning
/// helper with no phase call in its own body).
struct PropsConstruction {
    key: String,
    configured: bool,
    function: Option<usize>,
}

/// Every configured construction and the call graph that reaches it (§13).
#[derive(Default)]
struct PropsScan {
    file: String,
    module: Vec<String>,
    functions: Vec<PropsFunction>,
    constructions: Vec<PropsConstruction>,
    current_function: Vec<usize>,
    app_impl_modules: BTreeSet<String>,
}

impl PropsScan {
    fn module_path(&self, path: &str) -> String {
        if self.module.is_empty() {
            path.to_owned()
        } else {
            format!("{path}::{}", self.module.join("::"))
        }
    }

    fn current_function(&self) -> Option<usize> {
        self.current_function.last().copied()
    }

    fn add_function(&mut self, name: String, private: bool) -> usize {
        let id = self.functions.len();
        self.functions.push(PropsFunction {
            module: self.module_path(&self.file),
            name,
            private,
            calls: Vec::new(),
        });
        id
    }

    fn record_call(&mut self, name: &str) {
        if let Some(function) = self.current_function()
            && name != "new"
        {
            self.functions[function].calls.push(name.to_owned());
        }
    }

    fn record_path_call(&mut self, path: &syn::Path) {
        if let Some(name) = path.segments.last().map(|s| s.ident.to_string()) {
            self.record_call(&name);
        }
    }

    fn record_construction(&mut self, key: String, configured: bool) {
        self.constructions.push(PropsConstruction {
            key,
            configured,
            function: self.current_function(),
        });
    }

    fn construction_module(&self, construction: &PropsConstruction) -> String {
        construction
            .function
            .map(|f| self.functions[f].module.clone())
            .unwrap_or_else(|| self.file.clone())
    }

    /// The chain's methods innermost-first, and the expression it is built on.
    fn chain<'a>(expr: &'a syn::Expr, methods: &mut Vec<String>) -> &'a syn::Expr {
        let mut node = expr;
        while let syn::Expr::MethodCall(mc) = node {
            methods.push(mc.method.to_string());
            node = mc.receiver.as_ref();
        }
        methods.reverse();
        node
    }

    /// Whether the chain configures the component: "configuration beyond
    /// `new(id, …)`" (§13) is every builder call, and those come **before**
    /// the phase call — after `update`/`draw` the receiver is a `Response`,
    /// not the component.
    fn is_configured(methods: &[String]) -> bool {
        methods.first().is_some_and(|m| !is_phase_method(m))
    }

    /// Visit a method chain without visiting its base call as a second
    /// construction. The base call's arguments still need a normal visit so
    /// nested props (notably `FieldSpec::new(... TextInput::new(...))`) are
    /// discovered.
    fn visit_chain_children(&mut self, expr: &syn::Expr) {
        match expr {
            syn::Expr::MethodCall(mc) => {
                for arg in &mc.args {
                    syn::visit::Visit::visit_expr(self, arg);
                }
                self.visit_chain_children(mc.receiver.as_ref());
            }
            syn::Expr::Call(call) => {
                for arg in &call.args {
                    syn::visit::Visit::visit_expr(self, arg);
                }
            }
            other => syn::visit::Visit::visit_expr(self, other),
        }
    }

    fn resolve_calls(&self, caller: usize, name: &str) -> Vec<usize> {
        let module = &self.functions[caller].module;
        let mut scopes = Vec::new();
        let mut scope = module.clone();
        loop {
            scopes.push(scope.clone());
            let Some(at) = scope.rfind("::") else {
                break;
            };
            scope.truncate(at);
        }
        for scope in scopes {
            let found: Vec<_> = self
                .functions
                .iter()
                .enumerate()
                .filter(|(_, f)| f.module == scope && f.name == name)
                .map(|(i, _)| i)
                .collect();
            if !found.is_empty() {
                return found;
            }
        }
        // Qualified calls and methods can cross a sibling module boundary;
        // keep that fallback conservative by staying in this source file.
        self.functions
            .iter()
            .enumerate()
            .filter(|(_, f)| f.module.starts_with(&format!("{}::", self.file)) && f.name == name)
            .map(|(i, _)| i)
            .collect()
    }

    fn reaches(&self, from: usize, target: usize, seen: &mut BTreeSet<usize>) -> bool {
        if from == target {
            return true;
        }
        if !seen.insert(from) {
            return false;
        }
        for call in self.functions[from].calls.clone() {
            for next in self.resolve_calls(from, &call) {
                if self.reaches(next, target, seen) {
                    return true;
                }
            }
        }
        false
    }

    fn phase_roots(&self, module: &str, phase: &str) -> Vec<usize> {
        self.functions
            .iter()
            .enumerate()
            .filter(|(_, f)| f.module == module && f.name == phase)
            .map(|(i, _)| i)
            .collect()
    }

    fn enforces_phases(&self, module: &str) -> bool {
        if self.app_impl_modules.contains(module) {
            return true;
        }
        !self.phase_roots(module, "update").is_empty()
            && !self.phase_roots(module, "draw").is_empty()
    }
}

impl<'ast> syn::visit::Visit<'ast> for PropsScan {
    fn visit_item(&mut self, item: &'ast syn::Item) {
        // a test builds the same configured props many times **on purpose**:
        // that is the fixture, not the screen
        if is_cfg_test(item_attrs(item)) {
            return;
        }
        syn::visit::visit_item(self, item);
    }

    fn visit_item_mod(&mut self, item: &'ast syn::ItemMod) {
        self.module.push(item.ident.to_string());
        syn::visit::visit_item_mod(self, item);
        self.module.pop();
    }

    fn visit_item_fn(&mut self, item: &'ast syn::ItemFn) {
        let function = self.add_function(
            item.sig.ident.to_string(),
            matches!(item.vis, syn::Visibility::Inherited),
        );
        self.current_function.push(function);
        syn::visit::visit_item_fn(self, item);
        self.current_function.pop();
    }

    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        if item
            .trait_
            .as_ref()
            .is_some_and(|(_, path, _)| path.segments.last().is_some_and(|s| s.ident == "App"))
        {
            self.app_impl_modules.insert(self.module_path(&self.file));
        }
        syn::visit::visit_item_impl(self, item);
    }

    fn visit_impl_item_fn(&mut self, item: &'ast syn::ImplItemFn) {
        if is_cfg_test(&item.attrs) {
            return;
        }
        let function = self.add_function(
            item.sig.ident.to_string(),
            matches!(item.vis, syn::Visibility::Inherited),
        );
        self.current_function.push(function);
        syn::visit::visit_impl_item_fn(self, item);
        self.current_function.pop();
    }

    fn visit_expr(&mut self, expr: &'ast syn::Expr) {
        match expr {
            syn::Expr::MethodCall(_) => {
                let mut methods = Vec::new();
                let base = Self::chain(expr, &mut methods);
                if let Some(key) = ctor_const_key(base) {
                    self.record_construction(key, Self::is_configured(&methods));
                }
                // Method names are enough for the intra-file call graph and
                // also cover `self.props().draw(...)`, whose base is itself a
                // method chain rather than a free-function call.
                for method in &methods {
                    self.record_call(method);
                }
                if let syn::Expr::Call(call) = base
                    && let syn::Expr::Path(path) = call.func.as_ref()
                {
                    self.record_path_call(&path.path);
                }
                self.visit_chain_children(expr);
            }
            syn::Expr::Call(call) => {
                if let Some(key) = ctor_const_key(expr) {
                    self.record_construction(key, false);
                }
                if let syn::Expr::Path(path) = call.func.as_ref() {
                    self.record_path_call(&path.path);
                }
                for arg in &call.args {
                    self.visit_expr(arg);
                }
            }
            _ => syn::visit::visit_expr(self, expr),
        }
    }
}

/// One file: how many configured keyed constructions it holds, and every §13
/// violation among them — duplicate constructors, phase paths that do not
/// reach the one private constructor, or same-ID hand-rolled construction.
///
/// The count is returned because a check that observes nothing must not pass
/// (§47.4); it is not a diagnostic.
fn props_built_once_scan(path: &str, source: &str) -> (usize, Vec<String>) {
    let Ok(ast) = syn::parse_file(source) else {
        return (0, vec![format!("{path}: does not parse")]);
    };
    let mut scan = PropsScan {
        file: path.to_owned(),
        ..PropsScan::default()
    };
    syn::visit::Visit::visit_file(&mut scan, &ast);
    let observed = scan.constructions.iter().filter(|c| c.configured).count();
    let mut grouped: BTreeMap<(String, String), Vec<usize>> = BTreeMap::new();
    for (i, construction) in scan.constructions.iter().enumerate() {
        grouped
            .entry((
                scan.construction_module(construction),
                construction.key.clone(),
            ))
            .or_default()
            .push(i);
    }
    let mut hits = Vec::new();
    for ((module, key), indices) in grouped {
        let configured: Vec<_> = indices
            .iter()
            .copied()
            .filter(|i| scan.constructions[*i].configured)
            .collect();
        if configured.is_empty() {
            continue;
        }
        if configured.len() > 1 {
            hits.push(format!(
                "{module}: {key} is configured {} times; build it once in a private constructor called from both phases (§13)",
                configured.len()
            ));
            continue;
        }
        let ctor = configured[0];
        let Some(owner) = scan.constructions[ctor].function else {
            hits.push(format!(
                "{module}: {key} is configured outside a private constructor; build it once in a private constructor called from both phases (§13)"
            ));
            continue;
        };
        if scan.enforces_phases(&module) && !scan.functions[owner].private {
            hits.push(format!(
                "{module}: {key}'s props constructor is public; make it private and call it from both phases (§13)"
            ));
        }
        if scan.enforces_phases(&module) {
            for phase in ["update", "draw"] {
                let roots = scan.phase_roots(&scan.functions[owner].module, phase);
                if roots.is_empty()
                    || !roots
                        .iter()
                        .any(|root| scan.reaches(*root, owner, &mut BTreeSet::new()))
                {
                    hits.push(format!(
                        "{module}: {key}'s props constructor is not transitively reached from {phase}; call the same private constructor from both phases (§13)"
                    ));
                }
            }
        }
        if indices.iter().any(|i| *i != ctor) {
            hits.push(format!(
                "{module}: {key} is hand-rolled outside its single props constructor; route both phases through that constructor (§13)"
            ));
        }
    }
    (observed, hits)
}

/// The violations of [`props_built_once_scan`], for the red proof.
#[cfg(test)]
fn props_built_once_hits(path: &str, source: &str) -> Vec<String> {
    props_built_once_scan(path, source).1
}

/// §13 / §16.5 / §73. Props are built once.
///
/// A component instance configured beyond `X::new(id, …)` is built by exactly
/// one private constructor called from both phases, so a configured
/// construction keyed by a `const Id` appears **at most once** per module.
/// Two of them is the silent-bug class §13 exists to kill: a `disabled(…)`
/// applied in `draw` and forgotten in `update`, which no compiler can see.
///
/// The check is **non-vacuous by construction**: it fails when an existing
/// root holds no Rust file, and it fails when the whole scan observed no
/// configured keyed construction at all — a check that can only pass by
/// having nothing to look at is the §47.4 blind spot, not a guarantee.
fn props_are_built_once() -> Result<(), String> {
    let mut hits = Vec::new();
    let mut observed = 0usize;
    let mut scanned: Vec<(&str, usize)> = Vec::new();
    for root_name in PROPS_ROOTS {
        let dir = root().join(root_name);
        if !dir.exists() {
            continue;
        }
        let mut in_root = 0usize;
        for file in rust_files(&dir) {
            let path = rel(&file);
            // `apps/**/src` only: an application's own tests are not screens
            if root_name == "apps" && !path.contains("/src/") {
                continue;
            }
            in_root = in_root.saturating_add(1);
            let source = read(&file);
            let (found, violations) = props_built_once_scan(&path, &source);
            observed = observed.saturating_add(found);
            hits.extend(violations);
        }
        scanned.push((root_name, in_root));
    }
    let files: usize = scanned.iter().map(|(_, n)| *n).sum();
    hits.extend(props_vacuity_hits(&scanned, observed));
    println!(
        "props_are_built_once: {files} file(s) in {}, {observed} configured construction(s)",
        scanned
            .iter()
            .map(|(r, n)| format!("{r} ({n})"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    if hits.is_empty() {
        Ok(())
    } else {
        Err(hits.join("\n"))
    }
}

/// Why the scan may not be trusted: an existing root that yielded no Rust
/// file, or a whole scan that observed no configured keyed construction.
///
/// §47.4's blind spot is a check that passes because it was looking at
/// nothing. A root moving away, a scope typo, or the scan silently ceasing to
/// recognise a construction all end here rather than in a green `ok` line.
fn props_vacuity_hits(scanned: &[(&str, usize)], observed: usize) -> Vec<String> {
    let mut hits: Vec<String> = scanned
        .iter()
        .filter(|(_, n)| *n == 0)
        .map(|(r, _)| format!("{r}: exists and holds no Rust file in scope"))
        .collect();
    if observed == 0 {
        hits.push(
            "no configured `X::new(CONST_ID)` construction was observed in any scanned root: the check would pass by looking at nothing".to_owned(),
        );
    }
    hits
}

fn metadata() -> Result<cargo_metadata::Metadata, String> {
    cargo_metadata::MetadataCommand::new()
        .manifest_path(root().join("Cargo.toml"))
        .exec()
        .map_err(|e| e.to_string())
}

const LIB: &str = "junie-tui";
const DECLARED: [&str; 5] = [
    "ratatui-core",
    "ratatui-crossterm",
    "unicode-width",
    "unicode-segmentation",
    "bitflags",
];
/// §22.7 (2a): absent from the **entire** normal closure.
const FORBIDDEN_ANYWHERE: [&str; 5] = [
    "ratatui",
    "ratatui-widgets",
    "ratatui-macros",
    // (2b): these can only arrive through `ratatui-core` features we disable
    "critical-section",
    "palette",
];

/// §22.7 (2c): crossterm's own internals. They may appear **only beneath
/// `ratatui-crossterm`** — they are crossterm's choice, not ours, and §22.4's
/// decision is about *our* containers (enforced by forbidden-pattern rule 26
/// over our source).
const ONLY_UNDER_CROSSTERM: [&str; 8] = [
    "smallvec",
    "parking_lot",
    "parking_lot_core",
    "lock_api",
    "scopeguard",
    "libc",
    "mio",
    "signal-hook",
];

/// `cargo tree -p junie-tui -e normal` lines: `(name, version, features)`.
/// With `prune_crossterm`, the backend's own `crossterm` subtree is left out:
/// the architecture mandates `ratatui-crossterm`, and what crossterm pulls
/// beneath itself (`parking_lot` → `smallvec`, …) is not a choice of ours.
fn lib_tree(prune_crossterm: bool) -> Result<Vec<(String, String, BTreeSet<String>)>, String> {
    let mut args = vec![
        "tree", "-p", LIB, "-e", "normal", "--prefix", "none", "-f", "{p}\t{f}",
    ];
    if prune_crossterm {
        args.extend(["--prune", "crossterm"]);
    }
    let out = Command::new("cargo")
        .args(args)
        .current_dir(root())
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(format!(
            "cargo tree failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let mut rows = Vec::new();
    for line in text.lines() {
        let (pkg, feats) = line.split_once('\t').unwrap_or((line, ""));
        let mut parts = pkg.split_whitespace();
        let (Some(name), Some(version)) = (parts.next(), parts.next()) else {
            continue;
        };
        let feats: BTreeSet<String> = feats
            .split(',')
            .filter(|f| !f.is_empty())
            .map(str::to_owned)
            .collect();
        rows.push((
            name.to_owned(),
            version.trim_start_matches('v').to_owned(),
            feats,
        ));
    }
    Ok(rows)
}

fn dependency_graph_is_exactly_the_declared_set() -> Result<(), String> {
    let md = metadata()?;
    let lib = md
        .packages
        .iter()
        .find(|p| p.name.as_str() == LIB)
        .ok_or("no junie-tui package")?;
    let mut errors = Vec::new();
    // (1) direct normal deps
    let direct: BTreeSet<String> = lib
        .dependencies
        .iter()
        .filter(|d| d.kind == cargo_metadata::DependencyKind::Normal)
        .map(|d| d.name.clone())
        .collect();
    let want: BTreeSet<String> = DECLARED.iter().map(|s| (*s).to_owned()).collect();
    if direct != want {
        errors.push(format!("direct normal deps {direct:?} != {want:?}"));
    }
    // (2) the normal closure, resolved for this package alone (features are
    // unified per selected package, never across the legacy root package);
    // crossterm's own subtree is pruned, and crossterm must be reachable
    // only through ratatui-crossterm
    let tree = lib_tree(true)?;
    let closure: BTreeSet<String> = tree.iter().map(|(n, _, _)| n.clone()).collect();
    let full = lib_tree(false)?;
    let full_names: BTreeSet<String> = full.iter().map(|(n, _, _)| n.clone()).collect();
    if !full_names.contains("crossterm") {
        errors
            .push("crossterm is not in the closure at all (ratatui-crossterm missing?)".to_owned());
    }
    // (2a) + (2b): absent from the ENTIRE closure, not merely the pruned one
    for f in FORBIDDEN_ANYWHERE {
        if full_names.contains(f) {
            errors.push(format!("{f} is in the entire normal closure (2a/2b)"));
        }
    }
    // (2d): no direct `smallvec`, no direct `crossterm`
    if direct.contains("crossterm") {
        errors.push("direct crossterm dependency (2d)".to_owned());
    }
    if direct.contains("smallvec") {
        errors.push("direct smallvec dependency (2d)".to_owned());
    }
    if closure.contains("smallvec") {
        errors.push("smallvec survives the crossterm prune (2c/2d)".to_owned());
    }
    // (2c): every path to each of crossterm's internals passes through
    // `ratatui-crossterm`. Printed on success so the exception is visible.
    for name in ONLY_UNDER_CROSSTERM {
        if !full_names.iter().any(|n| n == name || n.starts_with(name)) {
            continue; // not in the closure at all: nothing to prove
        }
        match inverted_paths(name) {
            Err(e) => errors.push(format!("cargo tree --invert {name}: {e}")),
            Ok(paths) => {
                if paths.trim().is_empty() {
                    continue;
                }
                if !paths.contains("ratatui-crossterm") {
                    errors.push(format!(
                        "{name} is reachable without ratatui-crossterm (2c):\n{paths}"
                    ));
                } else {
                    println!("2c: {name} is reachable only beneath ratatui-crossterm");
                }
            }
        }
    }
    // (3) apps
    for p in &md.packages {
        if !["showcase", "tablepro", "jackin-preview"].contains(&p.name.as_str()) {
            continue;
        }
        let d: BTreeSet<String> = p
            .dependencies
            .iter()
            .filter(|d| d.kind == cargo_metadata::DependencyKind::Normal)
            .map(|d| d.name.clone())
            .collect();
        if d.len() != 1 || !(d.contains("junie-tui") || d.contains(LIB)) {
            errors.push(format!("{}: direct normal deps {d:?}", p.name));
        }
    }
    // (4) single versions inside the closure
    for name in ["unicode-width", "unicode-segmentation", "bitflags"] {
        let versions: BTreeSet<&str> = tree
            .iter()
            .filter(|(n, _, _)| n == name)
            .map(|(_, v, _)| v.as_str())
            .collect();
        if versions.len() > 1 {
            errors.push(format!(
                "{name} resolves to {versions:?} inside junie-tui's closure"
            ));
        }
    }
    // (5) ratatui-core features
    if let Some((_, _, feats)) = tree.iter().find(|(n, _, _)| n == "ratatui-core") {
        // `default` is ratatui-core's empty default set and carries nothing
        let feats: BTreeSet<String> = feats.iter().filter(|f| *f != "default").cloned().collect();
        let want: BTreeSet<String> = ["std", "underline-color"]
            .iter()
            .map(|s| (*s).to_owned())
            .collect();
        if feats != want {
            errors.push(format!("ratatui-core features {feats:?} != {want:?}"));
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

/// `cargo tree -p junie-tui -e normal --invert <crate>`, for §22.7 (2c).
fn inverted_paths(name: &str) -> Result<String, String> {
    let out = Command::new("cargo")
        .args([
            "tree", "-p", LIB, "-e", "normal", "--invert", name, "--prefix", "none",
        ])
        .current_dir(root())
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        // an absent crate is not an error for this assertion
        return Ok(String::new());
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn library_has_no_application_dependency() -> Result<(), String> {
    let md = metadata()?;
    let lib = md
        .packages
        .iter()
        .find(|p| p.name.as_str() == LIB)
        .ok_or("no junie-tui package")?;
    let bad: Vec<String> = lib
        .dependencies
        .iter()
        .filter(|d| {
            ["showcase", "tablepro", "jackin-preview", "junie-tui"].contains(&d.name.as_str())
        })
        .map(|d| d.name.clone())
        .collect();
    if bad.is_empty() {
        Ok(())
    } else {
        Err(format!("library depends on {bad:?}"))
    }
}

// ─────────────── the `apps/` boundary guards (§16.5, §47.5) ───────────────

/// One application package: its `[[bin]]` name, its `apps/` directory, its
/// `[lib]` target name and the slice that owns the migration (§47.1).
///
/// The binary name doubles as the package name: Appendix B.3 item 7 gives each
/// app a package with a `[lib]` (`showcase_app`, …) and a thin `[[bin]]` whose
/// `main` calls `<app>::run()`.
struct AppPackage {
    bin: &'static str,
    dir: &'static str,
    lib: &'static str,
    slice: &'static str,
}

/// The three applications, in the order §47.1 migrates them.
const APPS: [AppPackage; 3] = [
    AppPackage {
        bin: "showcase",
        dir: "apps/showcase",
        lib: "showcase_app",
        slice: "Slice 5",
    },
    AppPackage {
        bin: "tablepro",
        dir: "apps/tablepro",
        lib: "tablepro_app",
        slice: "Slice 6",
    },
    AppPackage {
        bin: "jackin-preview",
        dir: "apps/jackin-preview",
        lib: "jackin_app",
        slice: "Slice 7",
    },
];

/// The workspace's tooling package. Its binary is the checker itself, not a
/// shipped application, and is excluded from `binary_names_are_preserved` by
/// **package name** so that a second tooling binary still fails.
const TOOLING: &str = "xtask";

/// The `bin` target names of the **legacy root package**, as cargo resolves
/// them — which includes autodiscovery, not only explicit `[[bin]]` sections.
///
/// That distinction is load-bearing and was verified rather than assumed:
/// deleting `[[bin]] showcase` while `src/bin/showcase/main.rs` remains leaves
/// the target in place, so the slice index below correctly reports the
/// migration as *not started*. §47.1 requires the section **and** the
/// `src/bin/<app>/**` tree to go in the same commit, and this reads the same
/// thing cargo does.
///
/// Empty once the root manifest goes virtual (§47.1, the commit between
/// Slice 7 and Slice 8), which is the correct reading: no root package
/// declares no binaries, so every application is due.
fn root_package_bins(md: &cargo_metadata::Metadata) -> BTreeSet<String> {
    md.packages
        .iter()
        .filter(|p| p.manifest_path.parent() == Some(md.workspace_root.as_path()))
        .flat_map(|p| p.targets.iter())
        .filter(|t| t.kind.contains(&cargo_metadata::TargetKind::Bin))
        .map(|t| t.name.clone())
        .collect()
}

/// The applications whose migration has **started**, and which must therefore
/// be present in full.
///
/// The slice index is read off the tree rather than off a constant a builder
/// has to bump: §47.1 binds the root package to lose `[[bin]] X` **in the same
/// commit** that adds `apps/X`, so "the root no longer declares `[[bin]] X`"
/// is exactly "`apps/X` is due". A builder who drops the root binary without
/// adding the package fails here; one who adds the package without dropping
/// the root binary fails here *and* in `binary_names_are_preserved`, which
/// sees the duplicate.
fn due_apps(md: &cargo_metadata::Metadata) -> Vec<&'static AppPackage> {
    let root_bins = root_package_bins(md);
    APPS.iter().filter(|a| !root_bins.contains(a.bin)).collect()
}

fn root_package(md: &cargo_metadata::Metadata) -> Option<&cargo_metadata::Package> {
    md.packages
        .iter()
        .find(|package| package.manifest_path.parent() == Some(md.workspace_root.as_path()))
}

fn legacy_binary_source(app: &AppPackage) -> PathBuf {
    root()
        .join("src/bin")
        .join(app.bin.replace('-', "_"))
        .join("main.rs")
}

fn migrated_binary_source(app: &AppPackage) -> PathBuf {
    root().join(app.dir).join("src/main.rs")
}

#[derive(Debug)]
struct BinaryTarget {
    package: String,
    source: PathBuf,
}

fn binary_target_layout_hits(
    app: &AppPackage,
    root_owned: bool,
    root_package: Option<&str>,
    actual_package: &str,
    actual_source: &Path,
) -> Vec<String> {
    let expected_package = if root_owned {
        root_package
    } else {
        Some(app.bin)
    };
    let mut hits = Vec::new();
    if expected_package != Some(actual_package) {
        hits.push(format!(
            "`[[bin]] {}` belongs to package `{actual_package}` at the wrong migration boundary; expected {expected_package:?}",
            app.bin
        ));
    }
    let expected_source = if root_owned {
        legacy_binary_source(app)
    } else {
        migrated_binary_source(app)
    };
    if actual_source != expected_source {
        hits.push(format!(
            "`[[bin]] {}` source is `{}`, expected `{}`",
            app.bin,
            rel(actual_source),
            rel(&expected_source)
        ));
    }
    hits
}

/// Return the migration error for a missing `apps/` root. `None` is the one
/// accepted pre-Slice-5 state: the root package still owns every app binary.
fn missing_apps_for_due(due: &[&'static AppPackage]) -> Option<String> {
    if due.is_empty() {
        return None;
    }
    let expected = due
        .iter()
        .map(|app| format!("{}/src", app.dir))
        .collect::<Vec<_>>()
        .join(", ");
    Some(format!(
        "apps/ is missing while migrated application source is due: expected {expected} — the \
         no-app state is accepted only while the root package still declares every \
         application binary (§47.5)"
    ))
}

fn missing_application_manifest(app: &AppPackage) -> String {
    format!(
        "{} is missing for due application `{}` — the migrated app root must contain its \
         Cargo.toml manifest before boundary scanning can pass (§47.1/§47.5)",
        Path::new(app.dir).join("Cargo.toml").display(),
        app.bin
    )
}

/// Rust source files in the migrated application packages.
///
/// Before Slice 5 the accepted tree has no `apps/` directory and the root
/// package still owns all three binaries. That is an explicit, temporary
/// state — not an empty scan. Once any root binary is dropped, its application
/// source root is due and every due root must exist and contain Rust input.
/// Application directories that appear before their root binary is dropped are
/// rejected as staging, even when their contents happen to be clean.
fn application_source_files(md: &cargo_metadata::Metadata) -> Result<Vec<PathBuf>, String> {
    let r = root();
    let apps = r.join("apps");
    let due = due_apps(md);
    if !apps.exists() {
        if due.is_empty() {
            return Ok(Vec::new());
        }
        return match missing_apps_for_due(&due) {
            Some(error) => Err(error),
            None => Err("application migration due-set became empty during the scan".to_owned()),
        };
    }
    if !apps.is_dir() {
        return Err(format!(
            "application scan root {} is not a directory",
            rel(&apps)
        ));
    }

    let due_names: BTreeSet<&str> = due.iter().map(|a| a.dir).collect();
    let mut errors = Vec::new();
    let mut dirs = Vec::new();
    let entries =
        std::fs::read_dir(&apps).map_err(|e| format!("cannot enumerate {}: {e}", rel(&apps)))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("cannot read {} entry: {e}", rel(&apps)))?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(app) = APPS.iter().find(|a| r.join(a.dir) == path) else {
            errors.push(format!(
                "{} is not one of the declared application roots {:?}",
                rel(&path),
                APPS.iter().map(|a| a.dir).collect::<Vec<_>>()
            ));
            continue;
        };
        if !due_names.contains(app.dir) {
            errors.push(format!(
                "{} exists while the root package still declares `[[bin]] {}` — \
                 applications must be added atomically with removal of the root binary (§47.1)",
                app.dir, app.bin
            ));
        }
        if !path.join("Cargo.toml").is_file() {
            errors.push(missing_application_manifest(app));
        }
        let src = path.join("src");
        if !src.is_dir() {
            errors.push(format!(
                "{}/src is missing or not a directory for due application `{}`",
                app.dir, app.bin
            ));
            continue;
        }
        let mut files = rust_files(&src);
        files.sort();
        if files.is_empty() {
            errors.push(format!(
                "{}/src contains no Rust files; an application root cannot be a green-empty \
                 scan (§47.5)",
                app.dir
            ));
        }
        dirs.push((app.dir, files));
    }

    for app in &due {
        if !dirs.iter().any(|(dir, _)| *dir == app.dir) {
            errors.push(format!(
                "{}/src is missing for due application `{}` — the root package no longer \
                 declares `[[bin]] {}` (§47.5)",
                app.dir, app.bin, app.bin
            ));
        }
    }
    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }

    let mut files = dirs
        .into_iter()
        .flat_map(|(_, files)| files)
        .collect::<Vec<_>>();
    files.sort();
    if files.is_empty() {
        return Err(
            "apps/ exists but no due application source files were discovered; refusing a \
             green-empty boundary scan (§47.5)"
                .to_owned(),
        );
    }
    Ok(files)
}

/// One source line that matches an application-boundary pattern.
fn application_pattern_hits(path: &str, source: &str, patterns: &[(&str, &str)]) -> Vec<String> {
    let regexes = patterns
        .iter()
        .filter_map(|(name, pattern)| Regex::new(pattern).ok().map(|re| (*name, re)))
        .collect::<Vec<_>>();
    if regexes.len() != patterns.len() {
        return vec![format!(
            "{path}: application-boundary scanner has an invalid pattern"
        )];
    }
    non_test_lines(source)
        .into_iter()
        .filter_map(|(line_number, line)| {
            let code = code_line(line);
            let matched = regexes
                .iter()
                .filter(|(_, re)| re.is_match(code))
                .map(|(name, _)| *name)
                .collect::<Vec<_>>();
            if matched.is_empty() {
                None
            } else {
                Some(format!(
                    "{path}:{line_number}: {} — application code must use the public \
                     component/runtime API",
                    matched.join(", ")
                ))
            }
        })
        .collect()
}

const GENERIC_COMPONENT_COPY_PATTERNS: [(&str, &str); 4] = [
    ("fn render", r"\bfn\s+render\s*\("),
    ("Style::new()", r"\bStyle\s*::\s*new\s*\(\s*\)"),
    ("Block::default()", r"\bBlock\s*::\s*default\s*\(\s*\)"),
    ("buf.set_string", r"\b(?:buf|buffer)\s*\.\s*set_string\s*\("),
];

const DISPATCH_COPY_PATTERNS: [(&str, &str); 4] = [
    (".owns()", r"\.\s*owns\s*\("),
    (".locate", r"\.\s*locate\b"),
    ("scrollbar::id_for", r"\bscrollbar\s*::\s*id_for\b"),
    (".child()", r"\.\s*child\s*\("),
];

fn generic_component_copy_hits(path: &str, source: &str) -> Vec<String> {
    // Jackin's animated rain is the one documented app-owned renderer. Its
    // exception is path-specific and cannot widen to a module or directory.
    if path.trim_start_matches("./") == "apps/jackin-preview/src/rain.rs" {
        return Vec::new();
    }
    application_pattern_hits(path, source, &GENERIC_COMPONENT_COPY_PATTERNS)
}

fn owns_or_locate_hits(path: &str, source: &str) -> Vec<String> {
    application_pattern_hits(path, source, &DISPATCH_COPY_PATTERNS)
}

/// §16.5 / §47.5. Applications compose the library; they do not copy generic
/// renderers, style construction, block framing or raw cell painting.
fn no_generic_component_copies_in_applications() -> Result<(), String> {
    let md = metadata()?;
    let files = application_source_files(&md)?;
    let mut hits = Vec::new();
    for file in &files {
        let path = rel(file);
        hits.extend(generic_component_copy_hits(&path, &read(file)));
    }
    if hits.is_empty() {
        if files.is_empty() {
            println!(
                "no_generic_component_copies_in_applications: accepted temporary no-app state \
                 (root package still owns every application binary)"
            );
        } else {
            println!(
                "no_generic_component_copies_in_applications: {} application source file(s) \
                 scanned",
                files.len()
            );
        }
        Ok(())
    } else {
        Err(format!(
            "generic component-copy patterns are forbidden in applications (rain.rs is the \
             only named exception):\n{}",
            hits.join("\n")
        ))
    }
}

/// §16.5 / §47.5. Runtime dispatch owns focus, hit-testing and child routing;
/// application screens may not carry the retired `owns`/`locate` machinery.
fn no_owns_or_locate_in_applications() -> Result<(), String> {
    let md = metadata()?;
    let files = application_source_files(&md)?;
    let mut hits = Vec::new();
    for file in &files {
        let path = rel(file);
        hits.extend(owns_or_locate_hits(&path, &read(file)));
    }
    if hits.is_empty() {
        if files.is_empty() {
            println!(
                "no_owns_or_locate_in_applications: accepted temporary no-app state (root \
                 package still owns every application binary)"
            );
        } else {
            println!(
                "no_owns_or_locate_in_applications: {} application source file(s) scanned",
                files.len()
            );
        }
        Ok(())
    } else {
        Err(format!(
            "application dispatch copies are forbidden (`owns`, `locate`, `id_for`, `child`):\n{}",
            hits.join("\n")
        ))
    }
}

const SHOWCASE_PAGE_COUNT: usize = 22;

/// The page axes are parsed from production syntax, not inferred from any
/// identifier appearing in a helper. Every page must occur in the enum, the
/// complete `ALL` list, the literal navigation registry and the runtime
/// dispatch match.
#[derive(Debug, Default, Eq, PartialEq)]
struct ShowcasePageContract {
    enum_variants: Vec<String>,
    all_variants: Vec<String>,
    nav_variants: Vec<String>,
    dispatch_variants: Vec<String>,
}

fn ungroup_expr(expr: &syn::Expr) -> &syn::Expr {
    match expr {
        syn::Expr::Group(group) => ungroup_expr(&group.expr),
        syn::Expr::Paren(paren) => ungroup_expr(&paren.expr),
        syn::Expr::Reference(reference) => ungroup_expr(&reference.expr),
        _ => expr,
    }
}

fn page_variant_path(path: &syn::Path) -> Option<String> {
    let mut segments = path.segments.iter();
    let first = segments.next()?.ident.to_string();
    let second = segments.next()?.ident.to_string();
    if segments.next().is_some() || !matches!(first.as_str(), "PageId" | "Self") {
        return None;
    }
    Some(second)
}

fn page_variant_expr(expr: &syn::Expr, context: &str) -> Result<String, String> {
    let syn::Expr::Path(path) = ungroup_expr(expr) else {
        return Err(format!(
            "{context} must be a `PageId::Variant` or `Self::Variant` path"
        ));
    };
    page_variant_path(&path.path)
        .ok_or_else(|| format!("{context} must be a `PageId::Variant` or `Self::Variant` path"))
}

fn page_variant_array(expr: &syn::Expr, context: &str) -> Result<Vec<String>, String> {
    let syn::Expr::Array(array) = ungroup_expr(expr) else {
        return Err(format!("{context} must be a literal array"));
    };
    array
        .elems
        .iter()
        .enumerate()
        .map(|(index, item)| page_variant_expr(item, &format!("{context}[{index}]")))
        .collect()
}

fn nav_entries_expr(expr: &syn::Expr, context: &str) -> Result<Vec<String>, String> {
    let syn::Expr::Array(array) = ungroup_expr(expr) else {
        return Err(format!("{context} must be a literal entry array"));
    };
    let mut variants = Vec::with_capacity(array.elems.len());
    for (index, item) in array.elems.iter().enumerate() {
        let syn::Expr::Struct(entry) = ungroup_expr(item) else {
            return Err(format!(
                "{context}[{index}] must be a NavEntry struct with an `id` field"
            ));
        };
        let Some(id) = entry
            .fields
            .iter()
            .find(|field| matches!(&field.member, syn::Member::Named(name) if name == "id"))
        else {
            return Err(format!(
                "{context}[{index}] has no `id` field; a sentinel or helper is not a page"
            ));
        };
        variants.push(page_variant_expr(
            &id.expr,
            &format!("{context}[{index}].id"),
        )?);
    }
    Ok(variants)
}

fn cfg_test_only(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attribute| {
        if !attribute.path().is_ident("cfg") {
            return false;
        }
        let syn::Meta::List(meta) = &attribute.meta else {
            return false;
        };
        meta.tokens
            .to_string()
            .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
            .any(|token| token == "test")
    })
}

#[cfg(test)]
fn nav_entries_in_items(
    items: &[syn::Item],
    found: &mut Vec<(PathBuf, String, Vec<String>)>,
    module: &mut Vec<String>,
    path: &Path,
) -> Result<(), String> {
    for item in items {
        match item {
            syn::Item::Const(item)
                if item.ident == "NAV_ENTRIES" && !cfg_test_only(&item.attrs) =>
            {
                let name = if module.is_empty() {
                    "NAV_ENTRIES".to_owned()
                } else {
                    format!("{}::NAV_ENTRIES", module.join("::"))
                };
                found.push((
                    path.to_path_buf(),
                    name.clone(),
                    nav_entries_expr(&item.expr, &name)?,
                ));
            }
            syn::Item::Static(item)
                if item.ident == "NAV_ENTRIES" && !cfg_test_only(&item.attrs) =>
            {
                let name = if module.is_empty() {
                    "NAV_ENTRIES".to_owned()
                } else {
                    format!("{}::NAV_ENTRIES", module.join("::"))
                };
                found.push((
                    path.to_path_buf(),
                    name.clone(),
                    nav_entries_expr(&item.expr, &name)?,
                ));
            }
            syn::Item::Mod(item) if !cfg_test_only(&item.attrs) => {
                if let Some((_, inner)) = &item.content {
                    module.push(item.ident.to_string());
                    nav_entries_in_items(inner, found, module, path)?;
                    module.pop();
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// Extract the one authoritative showcase page registry from source files.
#[cfg(test)]
fn showcase_nav_registry(
    files: &[(PathBuf, String)],
) -> Result<(PathBuf, BTreeSet<String>), String> {
    let mut found = Vec::new();
    for (path, source) in files {
        let ast =
            syn::parse_file(source).map_err(|e| format!("{} does not parse: {e}", rel(path)))?;
        nav_entries_in_items(&ast.items, &mut found, &mut Vec::new(), path)?;
    }
    match found.as_slice() {
        [] => Err(
            "apps/showcase/src has no `NAV_ENTRIES` registry; showcase coverage cannot be \
             established"
                .to_owned(),
        ),
        [(_, name, variants)] if variants.is_empty() => Err(format!(
            "showcase registry `{name}` contains no `PageId` entries; refusing a \
             green-empty coverage scan"
        )),
        [(path, _name, variants)] => Ok((path.clone(), variants.iter().cloned().collect())),
        many => Err(format!(
            "showcase source contains {} `NAV_ENTRIES` registries ({:?}); \
             coverage requires exactly one authoritative page registry",
            many.len(),
            many.iter().map(|(_, name, _)| name).collect::<Vec<_>>()
        )),
    }
}

struct PageDispatchUses {
    variants: Vec<String>,
    matches: usize,
}

impl<'ast> syn::visit::Visit<'ast> for PageDispatchUses {
    fn visit_expr_match(&mut self, node: &'ast syn::ExprMatch) {
        self.matches = self.matches.saturating_add(1);
        for arm in &node.arms {
            let mut patterns = PagePatternUses {
                variants: Vec::new(),
                seen: BTreeSet::new(),
            };
            syn::visit::Visit::visit_pat(&mut patterns, &arm.pat);
            self.variants.extend(patterns.variants);
        }
        syn::visit::visit_expr_match(self, node);
    }
}

struct PagePatternUses {
    variants: Vec<String>,
    seen: BTreeSet<String>,
}

impl<'ast> syn::visit::Visit<'ast> for PagePatternUses {
    fn visit_pat(&mut self, node: &'ast syn::Pat) {
        if let syn::Pat::Path(path) = node
            && let Some(variant) = page_variant_path(&path.path)
            && self.seen.insert(variant.clone())
        {
            self.variants.push(variant);
        }
        syn::visit::visit_pat(self, node);
    }
}

fn is_page_id_type(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Path(path) if path.path.segments.last().is_some_and(|segment| segment.ident == "PageId"))
}

fn page_function_takes_page_id(function: &syn::ItemFn) -> bool {
    function.sig.inputs.iter().any(|input| match input {
        syn::FnArg::Typed(typed) => is_page_id_type(&typed.ty),
        syn::FnArg::Receiver(_) => false,
    })
}

fn collect_page_contract_items(
    items: &[syn::Item],
    contract: &mut ShowcasePageContract,
    nav_found: &mut usize,
    all_found: &mut usize,
    dispatch_found: &mut usize,
    path: &Path,
) -> Result<(), String> {
    for item in items {
        if cfg_test_only(item_attrs(item)) {
            continue;
        }
        match item {
            syn::Item::Enum(item) if item.ident == "PageId" => {
                if !contract.enum_variants.is_empty() {
                    return Err(format!(
                        "{} contains more than one production `PageId` enum",
                        rel(path)
                    ));
                }
                contract.enum_variants = item
                    .variants
                    .iter()
                    .map(|variant| variant.ident.to_string())
                    .collect();
            }
            syn::Item::Impl(item) if matches!(&*item.self_ty, syn::Type::Path(ty) if ty.path.segments.last().is_some_and(|segment| segment.ident == "PageId")) => {
                for impl_item in &item.items {
                    if let syn::ImplItem::Const(constant) = impl_item
                        && constant.ident == "ALL"
                        && !cfg_test_only(&constant.attrs)
                    {
                        *all_found = all_found.saturating_add(1);
                        if *all_found > 1 {
                            return Err("showcase contains more than one production `PageId::ALL`"
                                .to_owned());
                        }
                        contract.all_variants = page_variant_array(&constant.expr, "PageId::ALL")?;
                    }
                }
            }
            syn::Item::Const(item)
                if item.ident == "NAV_ENTRIES" && !cfg_test_only(&item.attrs) =>
            {
                *nav_found = nav_found.saturating_add(1);
                if *nav_found > 1 {
                    return Err(
                        "showcase contains more than one production `NAV_ENTRIES`".to_owned()
                    );
                }
                contract.nav_variants = nav_entries_expr(&item.expr, "NAV_ENTRIES")?;
            }
            syn::Item::Static(item)
                if item.ident == "NAV_ENTRIES" && !cfg_test_only(&item.attrs) =>
            {
                *nav_found = nav_found.saturating_add(1);
                if *nav_found > 1 {
                    return Err(
                        "showcase contains more than one production `NAV_ENTRIES`".to_owned()
                    );
                }
                contract.nav_variants = nav_entries_expr(&item.expr, "NAV_ENTRIES")?;
            }
            syn::Item::Fn(item) if item.sig.ident == "page" => {
                *dispatch_found = dispatch_found.saturating_add(1);
                if *dispatch_found > 1 {
                    return Err(
                        "showcase contains more than one production `page` dispatch function"
                            .to_owned(),
                    );
                }
                if !page_function_takes_page_id(item) {
                    return Err("showcase `page` dispatch must take a `PageId` argument".to_owned());
                }
                let mut uses = PageDispatchUses {
                    variants: Vec::new(),
                    matches: 0,
                };
                syn::visit::Visit::visit_block(&mut uses, &item.block);
                if uses.matches == 0 {
                    return Err(
                        "showcase `page` dispatch has no production match on PageId".to_owned()
                    );
                }
                contract.dispatch_variants = uses.variants;
            }
            syn::Item::Mod(item) => {
                if let Some((_, inner)) = &item.content {
                    collect_page_contract_items(
                        inner,
                        contract,
                        nav_found,
                        all_found,
                        dispatch_found,
                        path,
                    )?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn page_axis_set(label: &str, values: &[String]) -> Result<BTreeSet<String>, String> {
    if values.len() != SHOWCASE_PAGE_COUNT {
        return Err(format!(
            "showcase {label} has {} page(s), expected exactly {SHOWCASE_PAGE_COUNT}",
            values.len()
        ));
    }
    let set: BTreeSet<String> = values.iter().cloned().collect();
    if set.len() != SHOWCASE_PAGE_COUNT {
        return Err(format!(
            "showcase {label} contains duplicate page identities: {values:?}"
        ));
    }
    Ok(set)
}

fn validate_showcase_page_contract(contract: &ShowcasePageContract) -> Result<(), String> {
    let enum_set = page_axis_set("PageId", &contract.enum_variants)?;
    let all_set = page_axis_set("PageId::ALL", &contract.all_variants)?;
    let nav_set = page_axis_set("NAV_ENTRIES", &contract.nav_variants)?;
    let dispatch_set = page_axis_set("page dispatch", &contract.dispatch_variants)?;
    let mut errors = Vec::new();
    for (label, set) in [
        ("PageId::ALL", &all_set),
        ("NAV_ENTRIES", &nav_set),
        ("page dispatch", &dispatch_set),
    ] {
        if set != &enum_set {
            errors.push(format!(
                "showcase {label} does not exactly match PageId: expected {enum_set:?}, got {set:?}"
            ));
        }
    }
    for (label, values) in [
        ("PageId::ALL", &contract.all_variants),
        ("NAV_ENTRIES", &contract.nav_variants),
        ("page dispatch", &contract.dispatch_variants),
    ] {
        if values != &contract.enum_variants {
            errors.push(format!(
                "showcase {label} order differs from PageId declaration; dead or reordered pages are not accepted"
            ));
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

fn showcase_page_contract(files: &[(PathBuf, String)]) -> Result<ShowcasePageContract, String> {
    let mut contract = ShowcasePageContract::default();
    let mut nav_found = 0usize;
    let mut all_found = 0usize;
    let mut dispatch_found = 0usize;
    for (path, source) in files {
        let ast = syn::parse_file(source)
            .map_err(|error| format!("{} does not parse: {error}", rel(path)))?;
        collect_page_contract_items(
            &ast.items,
            &mut contract,
            &mut nav_found,
            &mut all_found,
            &mut dispatch_found,
            path,
        )?;
    }
    if nav_found == 0 {
        return Err("showcase source has no production `NAV_ENTRIES` registry".to_owned());
    }
    if all_found == 0 {
        return Err("showcase source has no production `PageId::ALL` list".to_owned());
    }
    if dispatch_found == 0 {
        return Err("showcase source has no production `page(PageId)` dispatch".to_owned());
    }
    validate_showcase_page_contract(&contract)?;
    Ok(contract)
}

fn page_name_variants(name: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let compact = name
        .chars()
        .flat_map(char::to_lowercase)
        .collect::<String>();
    out.insert(compact);
    let mut snake = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_ascii_uppercase() && i != 0 {
            snake.push('_');
        }
        snake.extend(c.to_lowercase());
    }
    out.insert(snake);
    out
}

/// Page source files reachable from the `PageId` variants in `NAV_ENTRIES`.
/// A page left on disk but removed from the registry is deliberately excluded.
fn showcase_page_sources(
    files: &[(PathBuf, String)],
    variants: &BTreeSet<String>,
) -> Result<Vec<(String, String)>, String> {
    let mut selected = Vec::new();
    let mut missing = Vec::new();
    for variant in variants {
        let candidates = page_name_variants(variant);
        let matches = files
            .iter()
            .filter(|(path, _)| {
                let path_text = path.to_string_lossy().replace('\\', "/");
                if !path_text.contains("/src/pages/") || path_text.ends_with("/pages/mod.rs") {
                    return false;
                }
                path.file_stem()
                    .and_then(|stem| stem.to_str())
                    .is_some_and(|stem| candidates.contains(stem))
            })
            .map(|(path, source)| (rel(path), source.clone()))
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            if matches.is_empty() {
                missing.push(variant.clone());
            } else {
                return Err(format!(
                    "showcase registry entry `{variant}` resolves to {} page source files; \
                     exactly one named page module is required",
                    matches.len()
                ));
            }
        } else {
            selected.extend(matches);
        }
    }
    if !missing.is_empty() {
        return Err(format!(
            "showcase registry entries {:?} have no reachable page source; \
             coverage cannot be proven",
            missing
        ));
    }
    selected.sort_by(|a, b| a.0.cmp(&b.0));
    selected.dedup_by(|a, b| a.0 == b.0);
    if selected.len() != variants.len() {
        return Err(format!(
            "showcase registry resolves {} page source file(s) for {} page identities; \
             every one of the exact {SHOWCASE_PAGE_COUNT} pages needs its own module",
            selected.len(),
            variants.len()
        ));
    }
    if selected.is_empty() {
        return Err(
            "showcase page registry resolved to no source files; refusing a green-empty \
             coverage scan"
                .to_owned(),
        );
    }
    // The app root owns the registry and its always-visible chrome (for
    // example the navigation list and help dialog), so those public
    // components are demonstrations too.
    if let Some((path, source)) = files
        .iter()
        .find(|(path, _)| rel(path) == "apps/showcase/src/app.rs")
    {
        selected.push((rel(path), source.clone()));
    }
    Ok(selected)
}

fn showcase_component_name(case: &str) -> Option<String> {
    let name = case.strip_suffix("Case")?;
    (name != "Probe").then(|| name.to_owned())
}

/// A production function body and its local calls. A component mention only
/// counts after the same page module's `Page::draw` or `Page::update` root
/// reaches this body. This deliberately ignores shared `pages/mod.rs` helpers,
/// imports, signatures, dead helpers and test items.
#[derive(Debug, Default)]
struct ShowcaseFunction {
    name: String,
    root_draw: bool,
    root_update: bool,
    calls: BTreeSet<String>,
    names: BTreeSet<String>,
}

struct ShowcaseBodyUses {
    calls: BTreeSet<String>,
    names: BTreeSet<String>,
}

impl<'ast> syn::visit::Visit<'ast> for ShowcaseBodyUses {
    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let syn::Expr::Path(path) = node.func.as_ref()
            && let Some(segment) = path.path.segments.last()
        {
            self.calls.insert(segment.ident.to_string());
        }
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        self.calls.insert(node.method.to_string());
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_path(&mut self, path: &'ast syn::Path) {
        self.names.extend(
            path.segments
                .iter()
                .map(|segment| segment.ident.to_string()),
        );
        syn::visit::visit_path(self, path);
    }
}

fn showcase_function_from_body(
    name: &syn::Ident,
    block: &syn::Block,
    root_draw: bool,
    root_update: bool,
) -> ShowcaseFunction {
    let mut uses = ShowcaseBodyUses {
        calls: BTreeSet::new(),
        names: BTreeSet::new(),
    };
    syn::visit::Visit::visit_block(&mut uses, block);
    ShowcaseFunction {
        name: name.to_string(),
        root_draw,
        root_update,
        calls: uses.calls,
        names: uses.names,
    }
}

fn collect_showcase_functions(items: &[syn::Item], functions: &mut Vec<ShowcaseFunction>) {
    for item in items {
        if cfg_test_only(item_attrs(item)) {
            continue;
        }
        match item {
            syn::Item::Fn(function) => functions.push(showcase_function_from_body(
                &function.sig.ident,
                &function.block,
                false,
                false,
            )),
            syn::Item::Impl(item_impl) => {
                let page_impl = item_impl
                    .trait_
                    .as_ref()
                    .and_then(|(_, path, _)| path.segments.last())
                    .is_some_and(|segment| segment.ident == "Page");
                for item in &item_impl.items {
                    let syn::ImplItem::Fn(function) = item else {
                        continue;
                    };
                    if cfg_test_only(&function.attrs) {
                        continue;
                    }
                    functions.push(showcase_function_from_body(
                        &function.sig.ident,
                        &function.block,
                        page_impl && function.sig.ident == "draw",
                        page_impl && function.sig.ident == "update",
                    ));
                }
            }
            syn::Item::Mod(item) => {
                if let Some((_, inner)) = &item.content {
                    collect_showcase_functions(inner, functions);
                }
            }
            _ => {}
        }
    }
}

fn showcase_page_path(path: &str) -> bool {
    let path = path.replace('\\', "/");
    path.contains("/src/pages/") && !path.ends_with("/src/pages/mod.rs")
}

fn showcase_component_names(pages: &[(String, String)]) -> Result<BTreeSet<String>, Vec<String>> {
    let mut reachable = BTreeSet::new();
    let mut errors = Vec::new();
    let mut page_count = 0usize;
    for (path, source) in pages {
        if !showcase_page_path(path) {
            continue;
        }
        page_count = page_count.saturating_add(1);
        let mut functions = Vec::new();
        match syn::parse_file(source) {
            Ok(ast) => {
                collect_showcase_functions(&ast.items, &mut functions);
                let has_draw = functions.iter().any(|function| function.root_draw);
                let has_update = functions.iter().any(|function| function.root_update);
                if !has_draw || !has_update {
                    errors.push(format!(
                        "{path}: every named page module must own production `Page::draw` and \
                         `Page::update`; a shared `pages/mod.rs` implementation cannot satisfy \
                         page coverage"
                    ));
                }
            }
            Err(error) => errors.push(format!("{path} does not parse: {error}")),
        }
        if errors.last().is_some_and(|error| error.starts_with(path)) {
            continue;
        }
        let mut pending = functions
            .iter()
            .filter(|function| function.root_draw || function.root_update)
            .map(|function| function.name.clone())
            .collect::<Vec<_>>();
        let mut visited = BTreeSet::new();
        let mut page_reachable = BTreeSet::new();
        while let Some(name) = pending.pop() {
            if !visited.insert(name.clone()) {
                continue;
            }
            for function in functions.iter().filter(|function| function.name == name) {
                page_reachable.extend(function.names.iter().cloned());
                pending.extend(function.calls.iter().cloned());
            }
        }
        if page_reachable.is_empty() {
            errors.push(format!(
                "{path}: production page roots reach no component usage; shared helpers outside \
                 this page cannot satisfy the roster"
            ));
        }
        reachable.extend(page_reachable);
    }
    if page_count == 0 {
        errors.push(
            "showcase has no named page modules with production Page roots; refusing shared-only \
             coverage"
                .to_owned(),
        );
    }
    if errors.is_empty() {
        Ok(reachable)
    } else {
        Err(errors)
    }
}

/// The pure coverage relation used by the filesystem check and its red-proof
/// fixtures. Every registered conformance case except the test-only probe must
/// occur in a page source reached through the page registry.
fn showcase_coverage_hits(
    cases: &BTreeSet<String>,
    registry_variants: &BTreeSet<String>,
    pages: &[(String, String)],
) -> Vec<String> {
    let mut hits = Vec::new();
    if registry_variants.is_empty() {
        hits.push("showcase page registry has no entries".to_owned());
    }
    if pages.is_empty() {
        hits.push("showcase page registry reaches no page source".to_owned());
    }
    let page_names = match showcase_component_names(pages) {
        Ok(names) => names,
        Err(errors) => {
            hits.extend(errors);
            BTreeSet::new()
        }
    };
    for case in cases {
        let Some(component) = showcase_component_name(case) else {
            continue;
        };
        if !page_names.contains(&component) {
            hits.push(format!(
                "{case}: no registered showcase page names public component `{component}`"
            ));
        }
    }
    hits
}

/// §16.5 / §47.5. The showcase page registry must reach every public
/// component represented by the shared conformance suite.
fn showcase_covers_every_public_component() -> Result<(), String> {
    let md = metadata()?;
    let due = due_apps(&md);
    let apps = root().join("apps");
    if !apps.exists() && due.is_empty() {
        println!(
            "showcase_covers_every_public_component: accepted temporary no-app state (root \
             package still owns every application binary)"
        );
        return Ok(());
    }
    if !due.iter().any(|a| a.bin == "showcase") {
        return Err(
            "showcase coverage is due only after the showcase binary leaves the root package; \
             an active apps/ tree without that atomic migration is invalid"
                .to_owned(),
        );
    }
    let _ = application_source_files(&md)?;
    let showcase_src = root().join("apps/showcase/src");
    let mut files = rust_files(&showcase_src)
        .into_iter()
        .map(|path| {
            let source = read(&path);
            (path, source)
        })
        .collect::<Vec<_>>();
    files.sort_by(|a, b| a.0.cmp(&b.0));
    let contract = showcase_page_contract(&files)?;
    let variants: BTreeSet<String> = contract.nav_variants.iter().cloned().collect();
    let pages = showcase_page_sources(&files, &variants)?;
    let cases = registered_conformance_cases(&root().join("crates/tui/tests/conformance.rs"))?;
    let hits = showcase_coverage_hits(&cases, &variants, &pages);
    if hits.is_empty() {
        println!(
            "showcase_covers_every_public_component: {} public conformance component(s) \
             reached through {} registered page(s)",
            cases
                .iter()
                .filter(|c| showcase_component_name(c).is_some())
                .count(),
            variants.len()
        );
        Ok(())
    } else {
        Err(hits.join("\n"))
    }
}

/// §16.5 / §47.5. The multiset of `bin` target names across every workspace
/// member equals `{showcase, tablepro, jackin-preview}`.
///
/// **An equality over a multiset, deliberately.** Containment would miss a
/// rename; a set would miss the migration's real hazard, which §47.5 states
/// as: the instant the root package and `apps/showcase` both declare
/// `[[bin]] showcase`, `target/debug/showcase` is whichever built last and the
/// capture harness silently captures the wrong program. That shows up only as
/// a *count* of two.
///
/// This check is meaningful on today's tree — the three names are the root
/// package's binaries — and stays meaningful after each migration, because the
/// name moves rather than changes.
fn binary_names_are_preserved() -> Result<(), String> {
    let md = metadata()?;
    let mut found: BTreeMap<String, Vec<BinaryTarget>> = BTreeMap::new();
    let mut tooling: Vec<String> = Vec::new();
    for p in md.workspace_packages() {
        for t in &p.targets {
            if !t.kind.contains(&cargo_metadata::TargetKind::Bin) {
                continue;
            }
            if p.name.as_str() == TOOLING {
                tooling.push(t.name.clone());
                continue;
            }
            found.entry(t.name.clone()).or_default().push(BinaryTarget {
                package: p.name.as_str().to_owned(),
                source: t.src_path.as_std_path().to_path_buf(),
            });
        }
    }
    let mut errors = Vec::new();
    let root_bins = root_package_bins(&md);
    let root_package = root_package(&md);
    if !root_bins.is_empty() && root_package.map(|p| p.name.as_str()) != Some("junie-tui") {
        errors.push(format!(
            "the legacy root package must remain named `junie-tui` while it owns binaries; found {:?}",
            root_package.map(|p| p.name.as_str())
        ));
    }
    // the tooling exclusion is by package, and the package is pinned to one bin
    if tooling != vec![TOOLING.to_owned()] {
        errors.push(format!(
            "the `{TOOLING}` package declares bins {tooling:?}, expected exactly [\"{TOOLING}\"] \
             — the tooling exclusion covers that one binary and nothing else"
        ));
    }
    for a in &APPS {
        match found.get(a.bin).map(Vec::as_slice) {
            None => errors.push(format!(
                "`[[bin]] {}` is missing from the workspace (owner {}): goal §21 preserves all \
                 three binary names across the split",
                a.bin, a.slice
            )),
            Some(targets) if targets.len() > 1 => errors.push(format!(
                "`[[bin]] {}` is declared by {} packages {:?} — `target/debug/{}` is whichever \
                 built last and the capture harness captures the wrong program (§47.5); §47.1 \
                 drops the root binary in the same commit that adds `{}`",
                a.bin,
                targets.len(),
                targets
                    .iter()
                    .map(|target| &target.package)
                    .collect::<Vec<_>>(),
                a.bin,
                a.dir
            )),
            Some([target]) => {
                errors.extend(binary_target_layout_hits(
                    a,
                    root_bins.contains(a.bin),
                    root_package.map(|p| p.name.as_str()),
                    &target.package,
                    &target.source,
                ));
            }
            Some(_) => {}
        }
    }
    let want: BTreeSet<&str> = APPS.iter().map(|a| a.bin).collect();
    for (name, pkgs) in &found {
        if !want.contains(name.as_str()) {
            errors.push(format!(
                "unexpected `[[bin]] {name}` in packages {:?}: the workspace ships exactly {want:?}",
                pkgs.iter().map(|target| &target.package).collect::<Vec<_>>()
            ));
        }
    }
    if errors.is_empty() {
        let where_from: Vec<String> = APPS
            .iter()
            .map(|a| {
                let pkg = found
                    .get(a.bin)
                    .and_then(|p| p.first())
                    .map_or("?", |target| target.package.as_str());
                format!("{}({pkg})", a.bin)
            })
            .collect();
        println!(
            "binary_names_are_preserved: {} — plus the `{TOOLING}` tooling binary",
            where_from.join(", ")
        );
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

/// §16.5 / §21 item 23 / §47.5. Every **due** application lib exists, is
/// `publish = false`, and no workspace member other than the library itself is
/// in the library's normal dependency closure.
///
/// **The expected set is slice-indexed and a missing member is a failure**
/// (§47.5): `{showcase_app}` from Slice 5, `tablepro_app` from 6,
/// `jackin_app` from 7, with `due_apps` reading the index off the root
/// package's remaining `[[bin]]`s rather than off a hand-maintained constant.
///
/// **Honest statement of what is vacuous today.** No `apps/` package exists,
/// so the expected set is empty and the *publish* and *lib-target* assertions
/// have nothing to run against; they are **vacuous until Slice 5**. Two halves
/// are not: the pairing assertion (a root `[[bin]]` dropped without its
/// package, or a package added while the root binary survives) is checkable on
/// today's tree, and the absent-from-closure half is asserted against **every
/// workspace member**, not only the three app libs — so the day the library
/// takes a path dependency on any member other than itself, this fails. That
/// generalisation is what makes the closure half demonstrable red today
/// instead of waiting for a package that does not exist.
fn app_libs_are_not_published_and_are_not_depended_on_by_the_library() -> Result<(), String> {
    let md = metadata()?;
    let root_bins = root_package_bins(&md);
    let members = md.workspace_packages();
    let mut errors = Vec::new();
    let mut present: Vec<&str> = Vec::new();
    for a in &APPS {
        let due = !root_bins.contains(a.bin);
        let pkg = members.iter().find(|p| p.name.as_str() == a.bin);
        match (due, pkg) {
            (true, None) => errors.push(format!(
                "{}: the root package no longer declares `[[bin]] {}`, so `{}` with `[lib] {}` is \
                 DUE and must exist — a missing expected member is a failure, not a pass (§47.5)",
                a.slice, a.bin, a.dir, a.lib
            )),
            (false, Some(_)) => errors.push(format!(
                "package `{}` exists while the root package still declares `[[bin]] {}`: §47.1 \
                 drops the root binary in the same commit that adds `{}`",
                a.bin, a.bin, a.dir
            )),
            (false, None) => {}
            (true, Some(p)) => {
                present.push(a.lib);
                let expected_manifest = root().join(a.dir).join("Cargo.toml");
                if p.manifest_path.as_std_path() != expected_manifest {
                    errors.push(format!(
                        "package `{}` is loaded from `{}`, expected `{}` (§47.1 app package layout)",
                        a.bin,
                        rel(p.manifest_path.as_std_path()),
                        rel(&expected_manifest)
                    ));
                }
                let libs: Vec<&cargo_metadata::Target> = p
                    .targets
                    .iter()
                    .filter(|t| t.kind.contains(&cargo_metadata::TargetKind::Lib))
                    .collect();
                if !p
                    .targets
                    .iter()
                    .any(|t| t.kind.contains(&cargo_metadata::TargetKind::Lib) && t.name == a.lib)
                {
                    errors.push(format!(
                        "package `{}` has lib target(s) {:?}, expected `[lib] {}` (§21 item \
                         23: the tests link the lib, so a binary-only package cannot host them)",
                        a.bin,
                        libs.iter()
                            .map(|target| target.name.as_str())
                            .collect::<Vec<_>>(),
                        a.lib
                    ));
                } else if libs.len() != 1 {
                    errors.push(format!(
                        "package `{}` declares {} library targets {:?}; expected exactly `[lib] {}`",
                        a.bin,
                        libs.len(),
                        libs.iter().map(|target| target.name.as_str()).collect::<Vec<_>>(),
                        a.lib
                    ));
                } else if let Some(lib) = libs.first() {
                    let expected_source = root().join(a.dir).join("src/lib.rs");
                    if lib.src_path.as_std_path() != expected_source {
                        errors.push(format!(
                            "package `{}` lib source is `{}`, expected `{}`",
                            a.bin,
                            rel(lib.src_path.as_std_path()),
                            rel(&expected_source)
                        ));
                    }
                }
                let bins: Vec<&cargo_metadata::Target> = p
                    .targets
                    .iter()
                    .filter(|t| {
                        t.kind.contains(&cargo_metadata::TargetKind::Bin) && t.name == a.bin
                    })
                    .collect();
                if bins.len() != 1 {
                    errors.push(format!(
                        "package `{}` has {} binary target(s) named `{}`; expected exactly one thin app binary",
                        a.bin,
                        bins.len(),
                        a.bin
                    ));
                } else if let Some(bin) = bins.first() {
                    let expected_source = migrated_binary_source(a);
                    if bin.src_path.as_std_path() != expected_source {
                        errors.push(format!(
                            "package `{}` binary source is `{}`, expected `{}`",
                            a.bin,
                            rel(bin.src_path.as_std_path()),
                            rel(&expected_source)
                        ));
                    }
                }
                if !p.publish.as_ref().is_some_and(Vec::is_empty) {
                    errors.push(format!(
                        "package `{}` has publish = {:?}, expected `publish = false` (§21 item 23)",
                        a.bin, p.publish
                    ));
                }
            }
        }
    }
    // the absent-from-closure half, generalised past the three app libs
    let closure: BTreeSet<String> = lib_tree(false)?.into_iter().map(|(n, _, _)| n).collect();
    let mut intruders: Vec<String> = Vec::new();
    for p in &members {
        let name = p.name.as_str();
        if name != LIB && closure.contains(name) {
            intruders.push(name.to_owned());
        }
    }
    for a in &APPS {
        if closure.contains(a.lib) {
            intruders.push(a.lib.to_owned());
        }
    }
    if !intruders.is_empty() {
        intruders.sort_unstable();
        intruders.dedup();
        errors.push(format!(
            "workspace member(s) {intruders:?} are in `{LIB}`'s normal dependency closure: the \
             library depends on no application and on no other workspace member (§16.5, §21 item \
             23)"
        ));
    }
    if errors.is_empty() {
        println!(
            "app_libs_are_not_published_and_are_not_depended_on_by_the_library: {} of {} app \
             lib(s) due and present {present:?}; {} workspace member(s) absent from `{LIB}`'s \
             normal closure",
            present.len(),
            APPS.len(),
            members.len().saturating_sub(1)
        );
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

/// Every root-level public name of the library: `pub mod` idents, the leaves
/// of the root's `pub use` trees, and every `#[macro_export]` macro (which
/// lives at the crate root wherever it is written).
///
/// Derived from the source rather than hard-coded, so the facade cannot grow a
/// module that this check silently blesses or forbids.
fn library_facade_roots() -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let lib = root().join("crates/tui/src/lib.rs");
    if let Ok(file) = syn::parse_file(&read(&lib)) {
        for item in &file.items {
            match item {
                syn::Item::Mod(m) if matches!(m.vis, syn::Visibility::Public(_)) => {
                    out.insert(m.ident.to_string());
                }
                syn::Item::Use(u) if matches!(u.vis, syn::Visibility::Public(_)) => {
                    use_leaves(&u.tree, &mut out);
                }
                _ => {}
            }
        }
    }
    let Ok(re) = Regex::new(r"macro_rules!\s*([A-Za-z_][A-Za-z0-9_]*)") else {
        return out;
    };
    for file in rust_files(&root().join("crates/tui/src")) {
        let text = read(&file);
        let lines: Vec<&str> = text.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if !line.trim_start().starts_with("#[macro_export]") {
                continue;
            }
            for l in lines.iter().skip(i.saturating_add(1)).take(4) {
                if let Some(c) = re.captures(l) {
                    if let Some(m) = c.get(1) {
                        out.insert(m.as_str().to_owned());
                    }
                    break;
                }
            }
        }
    }
    out
}

/// The names a `pub use` tree brings into the crate root.
fn use_leaves(tree: &syn::UseTree, out: &mut BTreeSet<String>) {
    match tree {
        syn::UseTree::Path(p) => use_leaves(&p.tree, out),
        syn::UseTree::Name(n) => {
            out.insert(n.ident.to_string());
        }
        syn::UseTree::Rename(r) => {
            out.insert(r.rename.to_string());
        }
        syn::UseTree::Group(g) => {
            for t in &g.items {
                use_leaves(t, out);
            }
        }
        syn::UseTree::Glob(_) => {}
    }
}

/// Collects the segment an application names **immediately after** the library
/// crate: `junie_tui::author::raw::Line` yields `author`, `use junie_tui::{Id,
/// author::Foo}` yields `Id` and `author`.
///
/// Token-based, never a regex: `use junie_tui::{A, b::C}` puts the interesting
/// idents inside a brace group that a regex over `crate::(\w+)` cannot see at
/// all.
struct FacadeUse {
    krate: &'static [&'static str],
    seen: BTreeSet<String>,
}

impl FacadeUse {
    fn use_tree(&mut self, tree: &syn::UseTree) {
        match tree {
            syn::UseTree::Path(p) => {
                if self.krate.contains(&p.ident.to_string().as_str()) {
                    self.after_crate(&p.tree);
                } else {
                    self.use_tree(&p.tree);
                }
            }
            syn::UseTree::Group(g) => {
                for t in &g.items {
                    self.use_tree(t);
                }
            }
            syn::UseTree::Name(_) | syn::UseTree::Rename(_) | syn::UseTree::Glob(_) => {}
        }
    }

    fn after_crate(&mut self, tree: &syn::UseTree) {
        match tree {
            syn::UseTree::Path(p) => {
                self.seen.insert(p.ident.to_string());
            }
            syn::UseTree::Name(n) => {
                self.seen.insert(n.ident.to_string());
            }
            syn::UseTree::Rename(r) => {
                self.seen.insert(r.ident.to_string());
            }
            syn::UseTree::Glob(_) => {
                self.seen.insert("*".to_owned());
            }
            syn::UseTree::Group(g) => {
                for t in &g.items {
                    self.after_crate(t);
                }
            }
        }
    }
}

impl syn::visit::Visit<'_> for FacadeUse {
    fn visit_item_use(&mut self, node: &syn::ItemUse) {
        self.use_tree(&node.tree);
        syn::visit::visit_item_use(self, node);
    }

    fn visit_path(&mut self, node: &syn::Path) {
        let mut it = node.segments.iter();
        if let (Some(first), Some(second)) = (it.next(), it.next())
            && self.krate.contains(&first.ident.to_string().as_str())
        {
            self.seen.insert(second.ident.to_string());
        }
        syn::visit::visit_path(self, node);
    }
}

/// The public library crate identifier used by every migrated consumer.
const LIB_CRATE_IDENTS: &[&str] = &["junie_tui"];

/// The first line of `text` naming `<crate>::<segment>`, 1-based.
fn first_facade_line(text: &str, segment: &str) -> usize {
    for ident in LIB_CRATE_IDENTS {
        let needle = format!("{ident}::{segment}");
        for (i, line) in text.lines().enumerate() {
            if line.contains(&needle) {
                return i.saturating_add(1);
            }
        }
    }
    for (i, line) in text.lines().enumerate() {
        if line.contains(segment) {
            return i.saturating_add(1);
        }
    }
    0
}

/// §16.5 / §47.5 / §47.8. The two thirds of "applications reach only the
/// public facade" that no other check performs.
///
/// **The third that is not here.** `cargo tree -p <app> -e normal --depth 1`
/// prints `junie-tui` and nothing else — that is asserted by
/// `dependency_graph_is_exactly_the_declared_set` item (3), whose app-package
/// loop already requires each app's direct normal dependencies to be exactly
/// the library. It is *not* duplicated here; duplicating it would produce two
/// checks that fail together and one that could quietly stop being run.
///
/// **What is here**: every path an application names under the library
/// resolves against the library's *root* public surface (`pub mod`s, root
/// `pub use` leaves, `#[macro_export]` macros — `author` and `author::raw`
/// among them), and no `#[path]` attribute or `include!` smuggles library
/// source into an application. §16.5 records that the *enforcement* is
/// structural — a separate crate cannot name a `pub(crate)` item — so this is
/// the belt-and-braces report that names the offender.
///
/// **Honest statement of what is vacuous today.** `apps/` does not exist, so
/// the path scan and the `#[path]`/`include!` prohibition have no input and
/// are **vacuous until Slice 5**; they were demonstrated red on a scratch
/// `apps/showcase` tree outside the repository, per `COORDINATION.md`. What is
/// not vacuous today is the due-set assertion: an application whose root
/// `[[bin]]` has been dropped must have an `apps/<app>/src` that this scan
/// actually read, so the check cannot pass by scanning nothing.
fn applications_depend_only_on_the_library_facade() -> Result<(), String> {
    let md = metadata()?;
    let due = due_apps(&md);
    let app_files = application_source_files(&md)?;
    let facade = library_facade_roots();
    let mut errors = Vec::new();
    let mut scanned = 0usize;
    let Ok(path_attr) = Regex::new(r"#\s*\[\s*path\s*=") else {
        return Err("bad #[path] regex".to_owned());
    };
    let Ok(include) = Regex::new(r"\binclude!\s*\(") else {
        return Err("bad include! regex".to_owned());
    };
    for file in &app_files {
        let text = read(file);
        let Ok(ast) = syn::parse_file(&text) else {
            errors.push(format!("{} does not parse", rel(file)));
            continue;
        };
        scanned = scanned.saturating_add(1);
        let mut visitor = FacadeUse {
            krate: LIB_CRATE_IDENTS,
            seen: BTreeSet::new(),
        };
        syn::visit::Visit::visit_file(&mut visitor, &ast);
        for segment in &visitor.seen {
            if facade.contains(segment) {
                continue;
            }
            errors.push(format!(
                "{}:{}: `{segment}` is not part of the library's root facade — an application \
                 names only the crate root or `author` (§16.5, §22 §1.2)",
                rel(file),
                first_facade_line(&text, segment)
            ));
        }
        for (n, line) in non_test_lines(&text) {
            let code = code_line(line);
            if path_attr.is_match(code) {
                errors.push(format!(
                    "{}:{n}: `#[path]` is forbidden in an application — a module reached by \
                     path is not a facade consumer (§16.5)",
                    rel(file)
                ));
            }
            if include.is_match(code) {
                errors.push(format!(
                    "{}:{n}: `include!` is forbidden in an application — included source \
                     bypasses the crate boundary the workspace exists to create (§16.5)",
                    rel(file)
                ));
            }
        }
    }
    if errors.is_empty() {
        println!(
            "applications_depend_only_on_the_library_facade: {} application source file(s) \
             scanned across {} due app(s) against {} facade root(s); the `cargo tree` third is \
             asserted by dependency_graph_is_exactly_the_declared_set (3), not duplicated here",
            scanned,
            due.len(),
            facade.len()
        );
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

fn grep_check(
    dirs: &[PathBuf],
    re: &str,
    allowed: &[&str],
    allow_file: Option<&str>,
    why: &str,
) -> Result<(), String> {
    let re = Regex::new(re).map_err(|e| e.to_string())?;
    let allow = allow_file.map(read_allow).unwrap_or_default();
    let mut hits = Vec::new();
    for dir in dirs {
        for file in rust_files(dir) {
            let path = rel(&file);
            if allowed.iter().any(|a| path.contains(a)) {
                continue;
            }
            for (ln, line) in non_test_lines(&read(&file)) {
                if re.is_match(code_line(line)) && !allow.contains_key(&format!("{path}:{ln}")) {
                    hits.push(format!("{path}:{ln}: {}", line.trim()));
                }
            }
        }
    }
    if hits.is_empty() {
        Ok(())
    } else {
        Err(format!("{why}\n{}", hits.join("\n")))
    }
}

/// Scans **code lines only**, deliberately (§4(j)-5): `\bworkspace\b` and
/// `\binstance\b` appear in ordinary architectural prose ("per-instance
/// patch"), and a reflowed `///` line must not fire the check. `grep_check`
/// strips comments through `code_line`.
fn no_domain_vocabulary_in_the_library() -> Result<(), String> {
    grep_check(
        &[root().join("crates/tui/src")],
        r"(?i)\b(sql|schema|primary key|nullable|foreign|references|not null|tablepro|jackin|workspace|instance|daemon|capsule|construct|catalog)\b",
        &[],
        Some("domain.txt"),
        "domain vocabulary in the library (DOM §7)",
    )
}

fn palette_literals_are_confined_to_theme_builtins() -> Result<(), String> {
    grep_check(
        &[root().join("crates/tui/src")],
        // §22.7's broad regex (D-10): the narrowed form let a computed
        // `Color::Rgb(r, g, b)` through anywhere.
        r"Color::Rgb\(|Color::from_u32\(|#[0-9a-fA-F]{6}\b",
        &[
            "theme/builtin/junie.rs",
            "theme/builtin/paper.rs",
            "theme/builder.rs",
            "theme/downgrade.rs",
        ],
        None,
        "colour literals outside theme/builtin (R-10)",
    )
}

/// BL-2: an "unreachable" arm implemented as `loop { spin_loop() }` hangs the
/// process with raw mode on and the alternate screen entered — strictly worse
/// than a panic, because `TerminalSession`'s hook can restore a panic but not
/// a livelock.
fn no_unreachable_spin_loops() -> Result<(), String> {
    grep_check(
        &[root().join("crates/tui/src")],
        r"spin_loop|loop\s*\{\s*\}",
        &[],
        None,
        "a livelock is not an `unreachable` arm (BL-2)",
    )
}

/// §22.1 as amended: `ratatui-crossterm` is a normal, non-optional dependency
/// taken for its version-unified `crossterm` **event vocabulary**, never for
/// `CrosstermBackend`. Exactly two files may name it: `event.rs` (the
/// vocabulary) and `runtime/session.rs` (the backend).
fn ratatui_crossterm_is_named_in_exactly_two_files() -> Result<(), String> {
    let re = Regex::new(r"ratatui_crossterm\b").map_err(|e| e.to_string())?;
    let mut files: Vec<String> = Vec::new();
    for file in rust_files(&root().join("crates/tui/src")) {
        if read(&file).lines().any(|l| re.is_match(l)) {
            files.push(rel(&file));
        }
    }
    files.sort();
    let want = [
        "crates/tui/src/event.rs".to_owned(),
        "crates/tui/src/runtime/session.rs".to_owned(),
    ];
    if files == want {
        println!("ratatui-crossterm is named in {files:?}");
        Ok(())
    } else {
        Err(format!(
            "ratatui_crossterm named in {files:?}, want {want:?}"
        ))
    }
}

fn no_raw_background_parameter() -> Result<(), String> {
    let re = Regex::new(r"\bbg:\s*(ratatui_core::style::)?Color\b").map_err(|e| e.to_string())?;
    let sig = Regex::new(r"\bpub (const )?fn\b").map_err(|e| e.to_string())?;
    let mut hits = Vec::new();
    for file in rust_files(&root().join("crates/tui/src")) {
        let path = rel(&file);
        for (ln, line) in non_test_lines(&read(&file)) {
            let code = code_line(line);
            if sig.is_match(code) && re.is_match(code) {
                hits.push(format!("{path}:{ln}: {}", line.trim()));
            }
        }
    }
    if hits.is_empty() {
        Ok(())
    } else {
        Err(hits.join("\n"))
    }
}

fn no_public_geometry_or_cache() -> Result<(), String> {
    grep_check(
        &[root().join("crates/tui/src/components")],
        r"\bpub (area|areas|anchor|scroll)\b|\bpub \w+_rects\b",
        &[],
        None,
        "public geometry on a component (S1)",
    )
}

fn no_fn_pointer_extension_points() -> Result<(), String> {
    grep_check(
        &[root().join("crates/tui/src")],
        r": fn\(|Option<fn\(|type \w+ = fn\(",
        &[],
        None,
        "fn-pointer extension point (API §3.12)",
    )
}

fn no_todo_or_unimplemented() -> Result<(), String> {
    let mut dirs = vec![root().join("crates")];
    if root().join("apps").exists() {
        dirs.push(root().join("apps"));
    }
    let re = Regex::new(r"todo!|unimplemented!|\bTODO\b|\bFIXME\b").map_err(|e| e.to_string())?;
    let mut hits = Vec::new();
    for dir in dirs {
        for file in rust_files(&dir) {
            let path = rel(&file);
            if path.contains("/target/") {
                continue;
            }
            for (i, line) in read(&file).lines().enumerate() {
                if re.is_match(line) && !line.contains("no_todo_or_unimplemented") {
                    hits.push(format!("{path}:{}: {}", i + 1, line.trim()));
                }
            }
        }
    }
    if hits.is_empty() {
        Ok(())
    } else {
        Err(hits.join("\n"))
    }
}

/// Whether a file carries the inner attribute `#![<level>(… <lint> …)]`.
///
/// Structural, because the substring form this replaced (`text.contains(
/// "#![forbid(unsafe_code)]")`) also matched the same characters inside a
/// line comment, a doc comment, a doc example or a string literal — a crate
/// that merely *documented* the attribute passed the gate. A `reason = "…"`
/// argument is consumed rather than aborting the walk, so it may precede the
/// lint.
fn file_has_inner_lint(path: &Path, level: &str, lint: &str) -> Result<bool, String> {
    let ast =
        syn::parse_file(&read(path)).map_err(|e| format!("{} does not parse: {e}", rel(path)))?;
    Ok(ast.attrs.iter().any(|a| {
        if !matches!(a.style, syn::AttrStyle::Inner(_)) || !a.path().is_ident(level) {
            return false;
        }
        let mut hit = false;
        let _ = a.parse_nested_meta(|m| {
            if m.path.is_ident(lint) {
                hit = true;
            }
            if m.input.peek(syn::Token![=]) {
                m.value()?.parse::<syn::Expr>()?;
            }
            Ok(())
        });
        hit
    }))
}

/// Whether any item — including items in inline modules — is an `unsafe impl`.
fn has_unsafe_impl(items: &[syn::Item]) -> bool {
    items.iter().any(|i| match i {
        syn::Item::Impl(im) => im.unsafety.is_some(),
        syn::Item::Mod(m) => m
            .content
            .as_ref()
            .is_some_and(|(_, inner)| has_unsafe_impl(inner)),
        _ => false,
    })
}

fn no_unsafe() -> Result<(), String> {
    let lib = root().join("crates/tui/src/lib.rs");
    if !file_has_inner_lint(&lib, "forbid", "unsafe_code")? {
        return Err("crates/tui/src/lib.rs lacks #![forbid(unsafe_code)]".to_owned());
    }
    let testing = root().join("crates/tui-testing/src/lib.rs");
    if !file_has_inner_lint(&testing, "deny", "unsafe_code")? {
        return Err("crates/tui-testing/src/lib.rs lacks #![deny(unsafe_code)]".to_owned());
    }
    let mut unsafe_files: Vec<String> = Vec::new();
    for f in rust_files(&root().join("crates/tui-testing/src")) {
        let ast =
            syn::parse_file(&read(&f)).map_err(|e| format!("{} does not parse: {e}", rel(&f)))?;
        if has_unsafe_impl(&ast.items) {
            unsafe_files.push(rel(&f));
        }
    }
    unsafe_files.sort();
    if unsafe_files == ["crates/tui-testing/src/perf.rs"] {
        Ok(())
    } else {
        Err(format!("unsafe impl in {unsafe_files:?}"))
    }
}

/// `syn`-based scans over `crates/tui/src` (and `components/**` when it exists).
fn parse_files(dir: &Path) -> Vec<(String, syn::File)> {
    rust_files(dir)
        .into_iter()
        .filter_map(|f| syn::parse_file(&read(&f)).ok().map(|ast| (rel(&f), ast)))
        .collect()
}

fn no_static_bound_in_component_surface() -> Result<(), String> {
    let dir = root().join("crates/tui/src/components");
    let mut hits = Vec::new();
    for (path, ast) in parse_files(&dir) {
        for item in &ast.items {
            if let syn::Item::Struct(s) = item
                && matches!(s.vis, syn::Visibility::Public(_))
            {
                let generics = quote_generics(&s.generics);
                if generics.contains("'static") {
                    hits.push(format!("{path}: {} has a 'static bound", s.ident));
                }
            }
        }
    }
    if hits.is_empty() {
        Ok(())
    } else {
        Err(hits.join("\n"))
    }
}

fn quote_generics(g: &syn::Generics) -> String {
    let mut s = String::new();
    for p in &g.params {
        if let syn::GenericParam::Type(t) = p {
            for b in &t.bounds {
                if let syn::TypeParamBound::Lifetime(l) = b {
                    s.push_str(&format!("'{}", l.ident));
                }
            }
        }
    }
    if let Some(w) = &g.where_clause {
        for pred in &w.predicates {
            if let syn::WherePredicate::Type(t) = pred {
                for b in &t.bounds {
                    if let syn::TypeParamBound::Lifetime(l) = b {
                        s.push_str(&format!("'{}", l.ident));
                    }
                }
            }
        }
    }
    s
}

fn draw_takes_shared_self() -> Result<(), String> {
    let dir = root().join("crates/tui/src/components");
    let mut hits = Vec::new();
    for (path, ast) in parse_files(&dir) {
        for item in &ast.items {
            if let syn::Item::Impl(im) = item {
                for it in &im.items {
                    if let syn::ImplItem::Fn(f) = it
                        && f.sig.ident == "draw"
                        && let Some(syn::FnArg::Receiver(r)) = f.sig.inputs.first()
                        && r.mutability.is_some()
                    {
                        hits.push(format!("{path}: fn draw takes &mut self"));
                    }
                }
            }
        }
    }
    if hits.is_empty() {
        Ok(())
    } else {
        Err(hits.join("\n"))
    }
}

struct ClosureDrawSpec<'a> {
    component: &'a str,
    path: &'a str,
    source: &'a str,
    state: Option<&'a str>,
    rects: usize,
}

fn path_is(ty: &syn::Type, want: &str) -> bool {
    matches!(
        ty,
        syn::Type::Path(p)
            if p.qself.is_none()
                && p.path.segments.len() == 1
                && p.path.segments.first().is_some_and(|s| s.ident == want)
    )
}

fn bare_path_is(ty: &syn::Type, want: &str) -> bool {
    matches!(
        ty,
        syn::Type::Path(p)
            if p.qself.is_none()
                && p.path.segments.len() == 1
                && p.path.segments.first().is_some_and(|s| {
                    s.ident == want && matches!(s.arguments, syn::PathArguments::None)
                })
    )
}

fn shared_self(arg: &syn::FnArg) -> bool {
    matches!(
        arg,
        syn::FnArg::Receiver(r)
            if r.reference.is_some() && r.mutability.is_none() && r.colon_token.is_none()
    )
}

fn reference_to(ty: &syn::Type, want: &str, mutable: bool) -> bool {
    matches!(
        ty,
        syn::Type::Reference(r)
            if r.mutability.is_some() == mutable && path_is(&r.elem, want)
    )
}

fn typed_arg_is(arg: &syn::FnArg, want: &str) -> bool {
    matches!(arg, syn::FnArg::Typed(p) if bare_path_is(&p.ty, want))
}

fn typed_arg_is_reference(arg: &syn::FnArg, want: &str, mutable: bool) -> bool {
    matches!(arg, syn::FnArg::Typed(p) if reference_to(&p.ty, want, mutable))
}

fn body_slot_has_shape(arg: &syn::FnArg, rects: usize) -> bool {
    let syn::FnArg::Typed(arg) = arg else {
        return false;
    };
    let syn::Pat::Ident(name) = arg.pat.as_ref() else {
        return false;
    };
    if name.ident != "body" {
        return false;
    }
    let syn::Type::ImplTrait(body) = arg.ty.as_ref() else {
        return false;
    };
    if body.bounds.len() != 1 {
        return false;
    }
    let Some(syn::TypeParamBound::Trait(bound)) = body.bounds.first() else {
        return false;
    };
    let Some(segment) = bound.path.segments.last() else {
        return false;
    };
    if bound.lifetimes.is_some()
        || !matches!(bound.modifier, syn::TraitBoundModifier::None)
        || bound.path.segments.len() != 1
        || segment.ident != "FnOnce"
    {
        return false;
    }
    let syn::PathArguments::Parenthesized(args) = &segment.arguments else {
        return false;
    };
    if args.inputs.len() != rects.saturating_add(1)
        || !args
            .inputs
            .first()
            .is_some_and(|ty| reference_to(ty, "Ui", true))
        || !args
            .inputs
            .iter()
            .skip(1)
            .all(|ty| bare_path_is(ty, "Rect"))
    {
        return false;
    }
    matches!(&args.output, syn::ReturnType::Type(_, ty) if bare_path_is(ty, "R"))
}

fn check_closure_draw_signature(spec: &ClosureDrawSpec<'_>) -> Result<(), String> {
    let ast =
        syn::parse_file(spec.source).map_err(|e| format!("{} does not parse: {e}", spec.path))?;
    let mut draws = Vec::new();
    for item in &ast.items {
        let syn::Item::Impl(item_impl) = item else {
            continue;
        };
        if item_impl.trait_.is_some() || !path_is(&item_impl.self_ty, spec.component) {
            continue;
        }
        for item in &item_impl.items {
            if let syn::ImplItem::Fn(method) = item
                && method.sig.ident == "draw"
                && matches!(method.vis, syn::Visibility::Public(_))
            {
                draws.push(method);
            }
        }
    }
    if draws.len() != 1 {
        return Err(format!(
            "{}: expected exactly one public {}::draw, found {}",
            spec.path,
            spec.component,
            draws.len()
        ));
    }
    let draw = draws[0];
    let generic_r = draw.sig.generics.params.len() == 1
        && draw.sig.generics.where_clause.is_none()
        && matches!(
            draw.sig.generics.params.first(),
            Some(syn::GenericParam::Type(param))
                if param.ident == "R" && param.bounds.is_empty() && param.default.is_none()
        );
    let bare_r = matches!(&draw.sig.output, syn::ReturnType::Type(_, ty) if bare_path_is(ty, "R"));
    let qualifiers_ok = draw.sig.constness.is_none()
        && draw.sig.asyncness.is_none()
        && draw.sig.unsafety.is_none()
        && draw.sig.abi.is_none()
        && draw.sig.variadic.is_none();
    let inputs: Vec<&syn::FnArg> = draw.sig.inputs.iter().collect();
    let state_args = usize::from(spec.state.is_some());
    let input_count = 4usize.saturating_add(state_args);
    let prefix_ok = inputs.len() == input_count
        && inputs.first().is_some_and(|arg| shared_self(arg))
        && inputs
            .get(1)
            .is_some_and(|arg| typed_arg_is_reference(arg, "Ui", true))
        && inputs.get(2).is_some_and(|arg| typed_arg_is(arg, "Rect"));
    let state_ok = spec.state.is_none_or(|state| {
        inputs
            .get(3)
            .is_some_and(|arg| typed_arg_is_reference(arg, state, false))
    });
    let body_index = 3usize.saturating_add(state_args);
    let body_ok = inputs
        .get(body_index)
        .is_some_and(|arg| body_slot_has_shape(arg, spec.rects));
    if qualifiers_ok && generic_r && bare_r && prefix_ok && state_ok && body_ok {
        Ok(())
    } else {
        Err(format!(
            "{}: {}::draw must be `pub fn draw<R>(&self, &mut Ui, Rect, {}body: impl FnOnce(&mut Ui, {}) -> R) -> R`",
            spec.path,
            spec.component,
            spec.state
                .map_or(String::new(), |state| format!("&{state}, ")),
            vec!["Rect"; spec.rects].join(", ")
        ))
    }
}

fn check_closure_draw_signatures(specs: &[ClosureDrawSpec<'_>]) -> Result<(), String> {
    let errors: Vec<String> = specs
        .iter()
        .filter_map(|spec| check_closure_draw_signature(spec).err())
        .collect();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

/// §§55–56: total closure-bearing containers return their body result without
/// optionality, and SplitPane is the sole two-Rect body-slot exception.
fn closure_bearing_draw_signatures_are_exact() -> Result<(), String> {
    let files = [
        ("Panel", "crates/tui/src/components/panel.rs", None, 1),
        (
            "Dialog",
            "crates/tui/src/components/dialog.rs",
            Some("DialogState"),
            1,
        ),
        (
            "SplitPane",
            "crates/tui/src/components/split.rs",
            Some("SplitPaneState"),
            2,
        ),
    ];
    let sources: Vec<String> = files
        .iter()
        .map(|(_, path, _, _)| read(&root().join(path)))
        .collect();
    let specs: Vec<ClosureDrawSpec<'_>> = files
        .iter()
        .zip(&sources)
        .map(
            |((component, path, state, rects), source)| ClosureDrawSpec {
                component,
                path,
                source,
                state: *state,
                rects: *rects,
            },
        )
        .collect();
    check_closure_draw_signatures(&specs)
}

fn path_with_infer_lifetime(ty: &syn::Type, want: &str) -> bool {
    let syn::Type::Path(path) = ty else {
        return false;
    };
    if path.qself.is_some() || path.path.segments.len() != 1 {
        return false;
    }
    let Some(segment) = path.path.segments.first() else {
        return false;
    };
    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
        return false;
    };
    segment.ident == want
        && args.args.len() == 1
        && matches!(args.args.first(), Some(syn::GenericArgument::Lifetime(l)) if l.ident == "_")
}

fn one_type_argument<'a>(ty: &'a syn::Type, outer: &str) -> Option<&'a syn::Type> {
    let syn::Type::Path(path) = ty else {
        return None;
    };
    if path.qself.is_some() || path.path.segments.len() != 1 {
        return None;
    }
    let segment = path.path.segments.first()?;
    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };
    if segment.ident != outer || args.args.len() != 1 {
        return None;
    }
    match args.args.first() {
        Some(syn::GenericArgument::Type(inner)) => Some(inner),
        _ => None,
    }
}

fn option_of(ty: &syn::Type, inner: impl FnOnce(&syn::Type) -> bool) -> bool {
    one_type_argument(ty, "Option").is_some_and(inner)
}

fn shared_str(ty: &syn::Type) -> bool {
    matches!(
        ty,
        syn::Type::Reference(r)
            if r.mutability.is_none() && r.lifetime.is_none() && bare_path_is(&r.elem, "str")
    )
}

fn shared_cell_actions(ty: &syn::Type) -> bool {
    matches!(
        ty,
        syn::Type::Reference(r)
            if r.mutability.is_none()
                && r.lifetime.is_none()
                && matches!(r.elem.as_ref(), syn::Type::Slice(s) if bare_path_is(&s.elem, "CellAction"))
    )
}

#[derive(Clone, Copy)]
enum GridReturn {
    Bare(&'static str),
    Inferred(&'static str),
    OptionalInferred(&'static str),
    OptionalStr,
    CellActions,
}

impl GridReturn {
    fn matches(self, output: &syn::ReturnType) -> bool {
        let syn::ReturnType::Type(_, ty) = output else {
            return false;
        };
        match self {
            GridReturn::Bare(name) => bare_path_is(ty, name),
            GridReturn::Inferred(name) => path_with_infer_lifetime(ty, name),
            GridReturn::OptionalInferred(name) => {
                option_of(ty, |inner| path_with_infer_lifetime(inner, name))
            }
            GridReturn::OptionalStr => option_of(ty, shared_str),
            GridReturn::CellActions => shared_cell_actions(ty),
        }
    }
}

struct GridMethodSpec {
    name: &'static str,
    usize_args: usize,
    output: GridReturn,
    default: bool,
}

const GRID_MODEL_METHODS: &[GridMethodSpec] = &[
    GridMethodSpec {
        name: "row_count",
        usize_args: 0,
        output: GridReturn::Bare("usize"),
        default: false,
    },
    GridMethodSpec {
        name: "row_key",
        usize_args: 1,
        output: GridReturn::Bare("ItemKey"),
        default: false,
    },
    GridMethodSpec {
        name: "cell",
        usize_args: 2,
        output: GridReturn::OptionalInferred("CellRef"),
        default: false,
    },
    GridMethodSpec {
        name: "row_decor",
        usize_args: 1,
        output: GridReturn::Inferred("RowDecor"),
        default: true,
    },
    GridMethodSpec {
        name: "cell_decor",
        usize_args: 2,
        output: GridReturn::Inferred("CellDecor"),
        default: true,
    },
    GridMethodSpec {
        name: "total",
        usize_args: 0,
        output: GridReturn::Bare("RowTotal"),
        default: true,
    },
    GridMethodSpec {
        name: "has_more",
        usize_args: 0,
        output: GridReturn::Bare("bool"),
        default: true,
    },
    GridMethodSpec {
        name: "read_only_reason",
        usize_args: 0,
        output: GridReturn::OptionalStr,
        default: true,
    },
    GridMethodSpec {
        name: "actions",
        usize_args: 2,
        output: GridReturn::CellActions,
        default: true,
    },
];

fn grid_method_matches(method: &syn::TraitItemFn, spec: &GridMethodSpec) -> bool {
    let inputs: Vec<&syn::FnArg> = method.sig.inputs.iter().collect();
    method.sig.generics.params.is_empty()
        && method.sig.generics.where_clause.is_none()
        && method.sig.constness.is_none()
        && method.sig.asyncness.is_none()
        && method.sig.unsafety.is_none()
        && method.sig.abi.is_none()
        && method.sig.variadic.is_none()
        && inputs.len() == spec.usize_args.saturating_add(1)
        && inputs.first().is_some_and(|arg| shared_self(arg))
        && inputs.iter().skip(1).all(|arg| typed_arg_is(arg, "usize"))
        && spec.output.matches(&method.sig.output)
        && method.default.is_some() == spec.default
}

fn check_grid_model_surface(source: &str, path: &str) -> Result<(), String> {
    let ast = syn::parse_file(source).map_err(|e| format!("{path} does not parse: {e}"))?;
    let traits: Vec<&syn::ItemTrait> = ast
        .items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Trait(item) if item.ident == "GridModel" => Some(item),
            _ => None,
        })
        .collect();
    if traits.len() != 1 {
        return Err(format!(
            "{path}: expected exactly one GridModel trait, found {}",
            traits.len()
        ));
    }
    let model = traits[0];
    let mut errors = Vec::new();
    if !matches!(model.vis, syn::Visibility::Public(_))
        || !model.generics.params.is_empty()
        || model.generics.where_clause.is_some()
        || !model.supertraits.is_empty()
    {
        errors.push("GridModel must be a non-generic public trait with no supertraits".to_owned());
    }
    let methods: Vec<&syn::TraitItemFn> = model
        .items
        .iter()
        .filter_map(|item| match item {
            syn::TraitItem::Fn(method) => Some(method),
            _ => None,
        })
        .collect();
    if methods.len() != model.items.len() {
        errors
            .push("GridModel may contain methods only; associated types are forbidden".to_owned());
    }
    let found: BTreeSet<String> = methods
        .iter()
        .map(|method| method.sig.ident.to_string())
        .collect();
    let expected: BTreeSet<String> = GRID_MODEL_METHODS
        .iter()
        .map(|method| method.name.to_owned())
        .collect();
    if found != expected || methods.len() != GRID_MODEL_METHODS.len() {
        errors.push(format!(
            "GridModel methods are {found:?}; expected exactly {expected:?} (no `col_count`)"
        ));
    }
    for spec in GRID_MODEL_METHODS {
        match methods.iter().find(|method| method.sig.ident == spec.name) {
            Some(method) if grid_method_matches(method, spec) => {}
            Some(_) => errors.push(format!(
                "GridModel::{} has wrong signature or required/default status",
                spec.name
            )),
            None => {}
        }
    }

    let cell_refs: Vec<&syn::ItemStruct> = ast
        .items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Struct(item) if item.ident == "CellRef" => Some(item),
            _ => None,
        })
        .collect();
    if cell_refs.len() != 1 {
        errors.push(format!(
            "expected exactly one CellRef struct, found {}",
            cell_refs.len()
        ));
    } else {
        let align: Vec<&syn::Field> = cell_refs[0]
            .fields
            .iter()
            .filter(|field| field.ident.as_ref().is_some_and(|ident| ident == "align"))
            .collect();
        if align.len() != 1
            || !matches!(align[0].vis, syn::Visibility::Public(_))
            || !option_of(&align[0].ty, |inner| bare_path_is(inner, "Align"))
        {
            errors.push("CellRef.align must be exactly `pub align: Option<Align>`".to_owned());
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(format!("{path}: {}", errors.join("\n")))
    }
}

/// §61: GridModel has one schema authority, structural ragged cells and the
/// exact default-hook set; CellRef alignment has an explicit inheritance bit.
fn grid_model_public_surface_is_exact() -> Result<(), String> {
    let path = "crates/tui/src/components/grid.rs";
    check_grid_model_surface(&read(&root().join(path)), path)
}

/// §67: FieldKind may borrow configured controls, but must not become generic
/// over application value/domain types.
fn field_kind_has_no_type_parameters() -> Result<(), String> {
    let path = "crates/tui/src/components/form.rs";
    let ast = syn::parse_file(&read(&root().join(path))).map_err(|error| error.to_string())?;
    let field_kind = ast.items.iter().find_map(|item| match item {
        syn::Item::Enum(item) if item.ident == "FieldKind" => Some(item),
        _ => None,
    });
    let Some(field_kind) = field_kind else {
        return Err(format!("{path}: missing public FieldKind enum"));
    };
    if !matches!(field_kind.vis, syn::Visibility::Public(_)) {
        return Err(format!("{path}: FieldKind must be public"));
    }
    let type_parameters = field_kind
        .generics
        .params
        .iter()
        .filter(|parameter| matches!(parameter, syn::GenericParam::Type(_)))
        .count();
    if type_parameters == 0 {
        Ok(())
    } else {
        Err(format!(
            "{path}: FieldKind has {type_parameters} type parameter(s); application domain types must remain in FormData"
        ))
    }
}

fn production_cache_types(text: &str) -> Result<BTreeSet<String>, String> {
    let re = Regex::new(r"cache::<(\w+)>").map_err(|e| e.to_string())?;
    let mut types = BTreeSet::new();
    for (_, line) in non_test_lines(text) {
        for captures in re.captures_iter(code_line(line)) {
            types.insert(captures[1].to_owned());
        }
    }
    Ok(types)
}

fn cache_types_are_derived_only() -> Result<(), String> {
    // constant pattern: built once, borrowed by the (cache type × file) loop below
    let state_struct =
        Regex::new(r"pub struct (\w+State)\s*\{([^}]*)\}").map_err(|e| e.to_string())?;
    let src = root().join("crates/tui/src");
    let mut cache_types = BTreeSet::new();
    let mut texts = Vec::new();
    for file in rust_files(&src) {
        let t = read(&file);
        cache_types.extend(production_cache_types(&t)?);
        texts.push((rel(&file), t));
    }
    let mut hits = Vec::new();
    for ty in &cache_types {
        let in_response = Regex::new(&format!(r"Response<{ty}>")).map_err(|e| e.to_string())?;
        let in_state = Regex::new(&format!(r"\b{ty}\b")).map_err(|e| e.to_string())?;
        for (path, t) in &texts {
            if in_response.is_match(t) {
                hits.push(format!("{path}: {ty} appears in a Response"));
            }
            // a struct named `*State` holding the cache type
            for m in state_struct.captures_iter(t) {
                if in_state.is_match(&m[2]) {
                    hits.push(format!("{path}: {} holds cache type {ty}", &m[1]));
                }
            }
        }
    }
    if hits.is_empty() {
        Ok(())
    } else {
        Err(hits.join("\n"))
    }
}

fn capability_has_no_unicode_field() -> Result<(), String> {
    let text = read(&root().join("crates/tui/src/theme/tokens.rs"));
    let ast = syn::parse_file(&text).map_err(|e| e.to_string())?;
    for item in ast.items {
        if let syn::Item::Struct(s) = item
            && s.ident == "Capability"
        {
            let fields: Vec<String> = s
                .fields
                .iter()
                .filter_map(|f| f.ident.as_ref().map(ToString::to_string))
                .collect();
            return if fields == ["color"] {
                Ok(())
            } else {
                Err(format!("Capability fields {fields:?}"))
            };
        }
    }
    Err("Capability not found".to_owned())
}

/// Finds a declared function of a given name anywhere in a file — free,
/// inherent, trait item or trait impl item.
struct FnNamed<'a> {
    want: &'a str,
    found: bool,
}

impl<'ast> syn::visit::Visit<'ast> for FnNamed<'_> {
    fn visit_signature(&mut self, sig: &'ast syn::Signature) {
        if sig.ident == self.want {
            self.found = true;
        }
        syn::visit::visit_signature(self, sig);
    }
}

/// Whether any item — including items in inline modules — declares the named
/// trait.
fn declares_trait(items: &[syn::Item], want: &str) -> bool {
    items.iter().any(|i| match i {
        syn::Item::Trait(t) => t.ident == want,
        syn::Item::Mod(m) => m
            .content
            .as_ref()
            .is_some_and(|(_, inner)| declares_trait(inner, want)),
        _ => false,
    })
}

fn no_boolean_capability_parameter_on_grid() -> Result<(), String> {
    let grid = root().join("crates/tui/src/components/grid.rs");
    if !grid.exists() {
        return Ok(());
    }
    // parsed, not grepped: `text.contains("fn editable(")` missed
    // `fn editable (`, `fn editable<T>(` and a signature broken across lines,
    // and matched a comment naming the forbidden shape
    let ast =
        syn::parse_file(&read(&grid)).map_err(|e| format!("{} does not parse: {e}", rel(&grid)))?;
    let mut hunt = FnNamed {
        want: "editable",
        found: false,
    };
    syn::visit::visit_file(&mut hunt, &ast);
    if hunt.found {
        return Err("crates/tui/src/components/grid.rs declares fn editable".to_owned());
    }
    for f in rust_files(&root().join("crates/tui/src")) {
        let ast =
            syn::parse_file(&read(&f)).map_err(|e| format!("{} does not parse: {e}", rel(&f)))?;
        if declares_trait(&ast.items, "GridCellActions") {
            return Err(format!("{}: trait GridCellActions", rel(&f)));
        }
    }
    Ok(())
}

fn core_is_backend_free() -> Result<(), String> {
    let status = Command::new("cargo")
        .args(["check", "-p", LIB, "--no-default-features", "-q"])
        .current_dir(root())
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err("cargo check --no-default-features failed".to_owned())
    }
}

fn msrv_and_edition_are_unchanged() -> Result<(), String> {
    let md = metadata()?;
    let mut errors = Vec::new();
    for p in &md.packages {
        if !md.workspace_members.contains(&p.id) {
            continue;
        }
        if p.edition != cargo_metadata::Edition::E2024 {
            errors.push(format!("{}: edition {:?}", p.name, p.edition));
        }
        let msrv = p
            .rust_version
            .as_ref()
            .map(|v| format!("{}.{}", v.major, v.minor));
        if msrv.as_deref() != Some("1.88") {
            errors.push(format!("{}: rust-version {:?}", p.name, p.rust_version));
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

// ───────────────────────────── doc-check ─────────────────────────────

/// Public items of the library: type/trait/fn/const/macro names, and
/// `Type::member` associations.
#[derive(Default, Debug)]
struct Api {
    items: BTreeSet<String>,
    members: BTreeSet<(String, String)>,
}

fn collect_api() -> Api {
    let mut api = Api::default();
    for (_, ast) in parse_files(&root().join("crates/tui/src")) {
        collect_items(&ast.items, &mut api);
    }
    collect_macro_generated(&mut api);
    api.items.insert("id".to_owned());
    api
}

/// Constants produced by the crate's declarative macros (`parts!`,
/// `action_keys!`, `newtype_u16!`, `bitflags!`), invisible to `syn`.
fn collect_macro_generated(api: &mut Api) {
    let const_re = Regex::new(r"(?m)^\s*([A-Z][A-Z0-9_]*)\s*=\s*[0-9]")
        .unwrap_or_else(|_| Regex::new("$^").unwrap_or_else(|_| unreachable!()));
    let flag_re = Regex::new(r"const ([A-Z][A-Z0-9_]*)\s*=")
        .unwrap_or_else(|_| Regex::new("$^").unwrap_or_else(|_| unreachable!()));
    // constant pattern: built once, borrowed by the `bitflags! {` scan below
    let struct_re = Regex::new(r"pub struct (\w+)")
        .unwrap_or_else(|_| Regex::new("$^").unwrap_or_else(|_| unreachable!()));
    for file in rust_files(&root().join("crates/tui/src")) {
        let text = read(&file);
        for (needle, ty) in [
            ("parts! {", Some("Part")),
            ("action_keys! {", Some("ActionKey")),
            ("newtype_u16! {", None),
        ] {
            let mut from = 0;
            while let Some(i) = text[from..].find(needle) {
                let start = from + i + needle.len();
                let end = text[start..].find("\n}").map_or(text.len(), |e| start + e);
                let body = &text[start..end];
                let ty = match ty {
                    Some(t) => t.to_owned(),
                    None => body
                        .lines()
                        .map(str::trim)
                        .find(|l| !l.starts_with("///") && !l.is_empty())
                        .and_then(|l| l.split(',').next())
                        .unwrap_or("")
                        .trim()
                        .to_owned(),
                };
                for cap in const_re.captures_iter(body) {
                    api.members.insert((ty.clone(), cap[1].to_owned()));
                }
                api.members.insert((ty.clone(), "ALL".to_owned()));
                api.members.insert((ty.clone(), "custom".to_owned()));
                api.members.insert((ty.clone(), "name".to_owned()));
                api.members.insert((ty.clone(), "raw".to_owned()));
                from = end;
            }
        }
        let mut from = 0;
        while let Some(i) = text[from..].find("bitflags! {") {
            let start = from + i;
            let end = text[start..].find("\n}").map_or(text.len(), |e| start + e);
            let body = &text[start..end];
            let ty = struct_re
                .captures(body)
                .map(|c| c[1].to_owned())
                .unwrap_or_default();
            for cap in flag_re.captures_iter(body) {
                api.members.insert((ty.clone(), cap[1].to_owned()));
            }
            for m in [
                "empty",
                "all",
                "bits",
                "contains",
                "iter",
                "iter_names",
                "from_bits_truncate",
                "union",
                "difference",
            ] {
                api.members.insert((ty.clone(), m.to_owned()));
            }
            from = end;
        }
    }
}

fn collect_items(items: &[syn::Item], api: &mut Api) {
    for item in items {
        match item {
            syn::Item::Struct(s) => {
                api.items.insert(s.ident.to_string());
                for f in &s.fields {
                    if let Some(id) = &f.ident {
                        api.members.insert((s.ident.to_string(), id.to_string()));
                    }
                }
            }
            syn::Item::Enum(e) => {
                api.items.insert(e.ident.to_string());
                for v in &e.variants {
                    api.members
                        .insert((e.ident.to_string(), v.ident.to_string()));
                }
            }
            syn::Item::Trait(t) => {
                api.items.insert(t.ident.to_string());
                for it in &t.items {
                    match it {
                        syn::TraitItem::Fn(f) => {
                            api.members
                                .insert((t.ident.to_string(), f.sig.ident.to_string()));
                        }
                        syn::TraitItem::Const(c) => {
                            api.members
                                .insert((t.ident.to_string(), c.ident.to_string()));
                        }
                        syn::TraitItem::Type(ty) => {
                            api.members
                                .insert((t.ident.to_string(), ty.ident.to_string()));
                        }
                        _ => {}
                    }
                }
            }
            syn::Item::Fn(f) => {
                api.items.insert(f.sig.ident.to_string());
            }
            syn::Item::Const(c) => {
                api.items.insert(c.ident.to_string());
            }
            syn::Item::Type(t) => {
                api.items.insert(t.ident.to_string());
            }
            syn::Item::Mod(m) => {
                api.items.insert(m.ident.to_string());
                if let Some((_, items)) = &m.content {
                    collect_items(items, api);
                }
            }
            syn::Item::Macro(m) => {
                if let Some(id) = &m.ident {
                    api.items.insert(id.to_string());
                }
            }
            syn::Item::Impl(im) => {
                let ty = match &*im.self_ty {
                    syn::Type::Path(p) => p.path.segments.last().map(|s| s.ident.to_string()),
                    _ => None,
                };
                let Some(ty) = ty else { continue };
                for it in &im.items {
                    match it {
                        syn::ImplItem::Fn(f) => {
                            api.members.insert((ty.clone(), f.sig.ident.to_string()));
                        }
                        syn::ImplItem::Const(c) => {
                            api.members.insert((ty.clone(), c.ident.to_string()));
                        }
                        _ => {}
                    }
                }
                // trait impls: the trait's methods become members of the type
                if let Some((_, path, _)) = &im.trait_
                    && let Some(seg) = path.segments.last()
                {
                    let tr = seg.ident.to_string();
                    for (t, m) in api.members.clone() {
                        if t == tr {
                            api.members.insert((ty.clone(), m));
                        }
                    }
                }
            }
            syn::Item::Use(u) => collect_use(&u.tree, api),
            _ => {}
        }
    }
}

fn collect_use(tree: &syn::UseTree, api: &mut Api) {
    match tree {
        syn::UseTree::Path(p) => collect_use(&p.tree, api),
        syn::UseTree::Name(n) => {
            api.items.insert(n.ident.to_string());
        }
        syn::UseTree::Rename(r) => {
            api.items.insert(r.rename.to_string());
        }
        syn::UseTree::Group(g) => {
            for t in &g.items {
                collect_use(t, api);
            }
        }
        syn::UseTree::Glob(_) => {}
    }
}

/// Well-known **foreign** members reachable through the facade's re-exports.
///
/// This table is for ratatui / crossterm API only. Legacy pre-refactor names
/// (`Theme::row`, `Theme::gutter`, `Interaction::pressed`,
/// `Interaction::focus_hidden`) used to be listed here as if they were
/// foreign API, which hid them; they are explicit entries in
/// `xtask/doc_check_allow.txt` instead (MA-14).
fn foreign_members() -> BTreeSet<(String, String)> {
    let mut m = BTreeSet::new();
    for (t, ms) in [
        (
            "Rect",
            &[
                "new",
                "clamp",
                "centered",
                "centered_horizontally",
                "centered_vertically",
                "inner",
                "outer",
                "intersection",
                "union",
                "offset",
                "is_empty",
                "ZERO",
                "positions",
                "rows",
                "columns",
            ][..],
        ),
        (
            "Color",
            &[
                "from_u32", "Rgb", "Reset", "Black", "DarkGray", "White", "Indexed",
            ][..],
        ),
        ("Style", &["new", "patch", "default"][..]),
        (
            "Buffer",
            &[
                "set_stringn",
                "set_line",
                "set_span",
                "set_string",
                "set_style",
                "cell",
                "cell_mut",
                "get",
                "get_mut",
                "empty",
            ][..],
        ),
        ("Frame", &["set_cursor_position", "area", "buffer_mut"][..]),
        (
            "KeyCode",
            &[
                "Char",
                "Enter",
                "Esc",
                "Tab",
                "Left",
                "Right",
                "Up",
                "Down",
                "BackTab",
                "Backspace",
                "Delete",
                "Home",
                "End",
                "PageUp",
                "PageDown",
                "F",
            ][..],
        ),
        ("KeyModifiers", &["CONTROL", "SHIFT", "ALT", "NONE"][..]),
        (
            "Modifier",
            &[
                "BOLD",
                "ITALIC",
                "UNDERLINED",
                "DIM",
                "REVERSED",
                "empty",
                "all",
            ][..],
        ),
        ("Position", &["new"][..]),
        ("Cell", &["symbol", "set_symbol", "set_style", "reset"][..]),
        ("Terminal", &["new", "draw", "try_draw"][..]),
        ("TestBackend", &["new"][..]),
        ("Vec", &["new"][..]),
        ("String", &["new"][..]),
        ("Option", &["None", "Some"][..]),
        ("Result", &["Ok", "Err"][..]),
        ("Default", &["default"][..]),
        ("mem", &["take"][..]),
        ("std", &["mem"][..]),
        ("core", &["fmt", "error"][..]),
        ("fmt", &["Debug", "Display", "Arguments"][..]),
        ("symbols", &["border", "line", "scrollbar"][..]),
        (
            "border",
            &["PLAIN", "ROUNDED", "DOUBLE", "ASCII", "Set"][..],
        ),
        ("scrollbar", &["Set", "VERTICAL"][..]),
        ("line", &["Set", "NORMAL", "THICK"][..]),
        (
            "ratatui_core",
            &["symbols", "buffer", "layout", "style", "text", "terminal"][..],
        ),
        ("ratatui_crossterm", &["crossterm", "CrosstermBackend"][..]),
        ("crossterm", &["event", "terminal"][..]),
        (
            "event",
            &[
                "KeyCode",
                "KeyModifiers",
                "EnableMouseCapture",
                "EnableBracketedPaste",
            ][..],
        ),
        (
            "terminal",
            &[
                "enable_raw_mode",
                "EnterAlternateScreen",
                "EnableLineWrap",
                "DisableLineWrap",
            ][..],
        ),
        ("Go", &["Manager", "Editor"][..]),
        ("Duration", &["from_millis"][..]),
        ("Cow", &["Borrowed", "Owned"][..]),
        (
            "MouseEventKind",
            &[
                "Moved",
                "Down",
                "Up",
                "Drag",
                "ScrollUp",
                "ScrollDown",
                "ScrollLeft",
                "ScrollRight",
            ][..],
        ),
        ("Event", &["as_key_press_event", "Key", "Mouse"][..]),
        ("KeyEvent", &["is_press", "is_repeat", "is_release"][..]),
        ("Constraint", &["Length", "Ratio"][..]),
        ("Margin", &["new"][..]),
        ("CellWidth", &["cell_width"][..]),
        ("Backend", &["Error", "size"][..]),
        ("Layout", &["split", "horizontal", "vertical"][..]),
        ("Terminal", &["insert_before"][..]),
        ("Style", &["underline_color"][..]),
        ("Self", &["PARTS", "State", "Action", "Cmd"][..]),
        ("Ident", &["method"][..]),
        ("PartEdit", &["when"][..]),
    ] {
        for x in ms {
            m.insert(((*t).to_owned(), (*x).to_owned()));
        }
    }
    m
}

/// One suppression in `xtask/doc_check_allow.txt`, retaining its source line
/// so stale and duplicate entries can be reported precisely.
#[derive(Debug, Clone, PartialEq, Eq)]
struct DocAllowEntry {
    name: String,
    line: usize,
}

/// Parse names of `junie_tui::` imports and `Type::member` references used in
/// the document's examples that are not yet built. The list is intentionally
/// a one-name-per-line format: deleting a line is the only normal way for it
/// to change as code lands.
fn parse_doc_allow(text: &str) -> Vec<DocAllowEntry> {
    text.lines()
        .enumerate()
        .filter_map(|(i, raw)| {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let entry = line.split('#').next().unwrap_or_default().trim();
            let name = entry.split_whitespace().next().unwrap_or_default();
            (!name.is_empty()).then_some(DocAllowEntry {
                name: name.to_owned(),
                line: i.saturating_add(1),
            })
        })
        .collect()
}

fn doc_allow_entries() -> Vec<DocAllowEntry> {
    parse_doc_allow(&read(&root().join("xtask/doc_check_allow.txt")))
}

/// A bare `Type` suppression covers `Type::member` references; a qualified
/// entry covers only itself. Every entry must cover at least one unresolved
/// reference in the current authoritative document, or it is stale.
fn allow_covers(entry: &str, hit: &str) -> bool {
    hit == entry
        || hit
            .strip_prefix(entry)
            .is_some_and(|rest| rest.starts_with("::"))
}

fn stale_doc_allow_entries(
    entries: &[DocAllowEntry],
    allowed_hits: &BTreeMap<String, usize>,
) -> Vec<String> {
    let mut issues = Vec::new();
    let mut seen: BTreeMap<&str, usize> = BTreeMap::new();
    for entry in entries {
        if let Some(first) = seen.insert(&entry.name, entry.line) {
            issues.push(format!(
                "duplicate entry `{}` at lines {first} and {}",
                entry.name, entry.line
            ));
        }
        if !allowed_hits
            .keys()
            .any(|hit| allow_covers(&entry.name, hit))
        {
            issues.push(format!(
                "stale entry `{}` at line {}: it suppresses no unresolved reference",
                entry.name, entry.line
            ));
        }
    }
    issues
}

/// Parse a top-level architecture heading. Historical headings use both
/// `## 27.` and `## §27`; the section marker is decoration, not part of the
/// number. Subheadings and Appendix headings are deliberately excluded.
fn top_level_doc_section_number(line: &str) -> Option<u32> {
    let mut rest = line.strip_prefix("## ")?;
    if let Some(number) = rest.strip_prefix('§') {
        rest = number;
    }
    let digits = rest.bytes().take_while(|b| b.is_ascii_digit()).count();
    if digits == 0 {
        return None;
    }
    let number = rest.get(..digits)?.parse().ok()?;
    let after = rest.get(digits..).unwrap_or_default();
    if after.is_empty()
        || after.starts_with('.')
        || after
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_whitespace())
    {
        Some(number)
    } else {
        None
    }
}

fn doc_sections(text: &str) -> String {
    // §3–§17 and §21 through the authoritative tail. The latter is open-ended
    // because this document records numbered adjudications over time; a fixed
    // upper bound is the scope hole that made §27–§72 invisible.
    let mut out = String::new();
    let mut keep = false;
    for line in text.lines() {
        if let Some(n) = top_level_doc_section_number(line) {
            // §24–§26 carry the M-, K- and correction-pass amendments; the
            // old range stopped at §23 and left them unchecked (MA-14, F23).
            keep = matches!(n, 3..=17 | 21..=u32::MAX);
        }
        if keep {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

fn doc_check() -> Result<(), String> {
    let text = read(&root().join("COMPONENT_ARCHITECTURE.md"));
    let scoped = doc_sections(&text);
    let api = collect_api();
    let foreign = foreign_members();
    let allow_entries = doc_allow_entries();
    let allow: BTreeSet<String> = allow_entries.iter().map(|e| e.name.clone()).collect();
    let member_re = Regex::new(r"`([A-Z][A-Za-z0-9_]*)::([A-Za-z_][A-Za-z0-9_]*)`")
        .map_err(|e| e.to_string())?;
    let mut unresolved: BTreeMap<String, usize> = BTreeMap::new();
    let mut allowed_hits: BTreeMap<String, usize> = BTreeMap::new();
    let mut resolved = 0usize;
    // `Type::member` references in prose
    for cap in member_re.captures_iter(&scoped) {
        let (t, m) = (cap[1].to_owned(), cap[2].to_owned());
        let key = format!("{t}::{m}");
        let ok = api.members.contains(&(t.clone(), m.clone()))
            || foreign.contains(&(t.clone(), m.clone()))
            || (api.items.contains(&t)
                && api.items.contains(&m)
                && t.chars().next().is_some_and(char::is_lowercase));
        if ok {
            resolved += 1;
        } else if allow.contains(&t) || allow.contains(&key) {
            *allowed_hits.entry(key).or_insert(0) += 1;
        } else {
            *unresolved.entry(key).or_insert(0) += 1;
        }
    }
    // `use junie_tui::{…}` lists in rust blocks
    let block_re = Regex::new(r"(?s)```rust\n(.*?)```").map_err(|e| e.to_string())?;
    let use_re =
        Regex::new(r"use junie_tui(?:::author)?::\{([^}]*)\}").map_err(|e| e.to_string())?;
    let use_one = Regex::new(r"use junie_tui(?:::author)?::([A-Za-z_][A-Za-z0-9_]*);")
        .map_err(|e| e.to_string())?;
    let mut blocks = 0usize;
    for b in block_re.captures_iter(&scoped) {
        blocks += 1;
        let body = &b[1];
        let mut names: Vec<String> = Vec::new();
        for u in use_re.captures_iter(body) {
            names.extend(
                u[1].split(',')
                    .map(|s| s.trim().trim_start_matches("self").trim_start_matches("::"))
                    .filter(|s| !s.is_empty())
                    .map(|s| s.split("::").last().unwrap_or(s).to_owned()),
            );
        }
        for u in use_one.captures_iter(body) {
            names.push(u[1].to_owned());
        }
        for n in names {
            if n == "id" || n == "self" || api.items.contains(&n) {
                resolved += 1;
            } else if allow.contains(&n) {
                *allowed_hits.entry(n).or_insert(0) += 1;
            } else {
                *unresolved.entry(n).or_insert(0) += 1;
            }
        }
    }
    println!(
        "doc-check: {blocks} rust blocks, {resolved} references resolved against crates/tui/src"
    );
    if !allowed_hits.is_empty() {
        println!("doc-check: not yet built (allow-listed, xtask/doc_check_allow.txt):");
        for (k, n) in &allowed_hits {
            println!("  {k} ({n})");
        }
    }
    let allow_issues = stale_doc_allow_entries(&allow_entries, &allowed_hits);
    if unresolved.is_empty() {
        if allow_issues.is_empty() {
            Ok(())
        } else {
            Err(format!(
                "doc-check: stale or duplicate allow-list entries:\n  {}",
                allow_issues.join("\n  ")
            ))
        }
    } else {
        let mut msg =
            String::from("doc-check: unresolved references (build them or allow-list them):\n");
        for (k, n) in &unresolved {
            msg.push_str(&format!("  {k} ({n})\n"));
        }
        if !allow_issues.is_empty() {
            msg.push_str("doc-check: stale or duplicate allow-list entries:\n");
            for issue in &allow_issues {
                msg.push_str(&format!("  {issue}\n"));
            }
        }
        Err(msg)
    }
}

// ─────────────── bless-guard (§16.3, §20.10, §36.5) ───────────────

/// The kind of baseline a path is, which fixes how its lines become keys.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum BaselineKind {
    /// `name w h theme color hash`. The key is the line minus its **last**
    /// whitespace field — exactly `Baseline::parse`'s rule in
    /// `crates/tui-testing/src/digest.rs`. Any other rule makes the guard and
    /// the writer disagree about what a key is.
    Digest,
    /// `name ns allocs bytes [hits ring]`. The key is the name and the compared
    /// value **excludes the `ns` column**: `crates/tui/tests/perf_baseline.txt`
    /// records that timings are re-measured per machine and asserted only under
    /// `PERF_STRICT`, so a guard that fired on timing noise would be suppressed,
    /// and a suppressed guard is a dead one.
    Perf,
    /// Frozen pre-refactor evidence (`baseline/before/**`, `tests/baselines/**`,
    /// `tests/showcase_baseline.txt`, the root `tests/perf_baseline.txt`). Never
    /// parsed into keys: any change fails, and the remedy is `git checkout --`,
    /// not a ledger entry. Much of it is binary, so change is read from git's
    /// name-status rather than from a text comparison.
    Frozen,
}

/// Which baseline rule a repository-relative path obeys, or `None` if it is not
/// a baseline at all. Patterns, never a hard-coded file list:
/// `apps/*/tests/baselines/*.txt` do not exist yet and arrive in Slices 5–7.
fn classify_baseline(rel: &str) -> Option<BaselineKind> {
    let frozen = rel.starts_with("baseline/before/")
        || rel == "tests/showcase_baseline.txt"
        || rel == "tests/perf_baseline.txt"
        || (rel.starts_with("tests/baselines/") && rel.ends_with(".txt"));
    if frozen {
        return Some(BaselineKind::Frozen);
    }
    if rel.ends_with("/tests/perf_baseline.txt") {
        return Some(BaselineKind::Perf);
    }
    if rel.ends_with(".txt") && rel.contains("/tests/baselines/") {
        return Some(BaselineKind::Digest);
    }
    None
}

/// A baseline file's `key -> compared value` map under its own rule. Blank and
/// `#` lines are skipped, as `Baseline::parse` skips them.
fn parse_baseline_entries(kind: BaselineKind, text: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        match kind {
            BaselineKind::Digest => {
                if let Some((key, hash)) = line.rsplit_once(' ') {
                    out.insert(key.to_owned(), hash.to_owned());
                }
            }
            BaselineKind::Perf => {
                let mut fields = line.split_whitespace();
                let Some(name) = fields.next() else { continue };
                let _ns = fields.next(); // re-measured per machine; never compared
                out.insert(name.to_owned(), fields.collect::<Vec<&str>>().join(" "));
            }
            // frozen evidence is compared as a whole file, never by key
            BaselineKind::Frozen => {}
        }
    }
    out
}

/// A baseline key whose recorded value changed.
#[derive(Debug, PartialEq, Eq)]
struct Movement {
    file: String,
    key: String,
    old: String,
    new: String,
}

/// A baseline key recorded for the first time.
#[derive(Debug, PartialEq, Eq)]
struct Addition {
    file: String,
    key: String,
}

/// The moved and added sets of one baseline file. A key present in the base and
/// absent now is a *removal*, which §36.5 does not make classifiable and which
/// this therefore does not report.
fn diff_baseline(
    file: &str,
    kind: BaselineKind,
    base: &str,
    work: &str,
) -> (Vec<Movement>, Vec<Addition>) {
    let before = parse_baseline_entries(kind, base);
    let after = parse_baseline_entries(kind, work);
    let mut moved = Vec::new();
    let mut added = Vec::new();
    for (key, value) in &after {
        match before.get(key) {
            Some(old) if old == value => {}
            Some(old) => moved.push(Movement {
                file: file.to_owned(),
                key: key.clone(),
                old: old.clone(),
                new: value.clone(),
            }),
            None => added.push(Addition {
                file: file.to_owned(),
                key: key.clone(),
            }),
        }
    }
    (moved, added)
}

/// What a §20.10 row's `{scope: …}` tag says the item is allowed to account
/// for (§49.3).
///
/// **Fail-closed.** The absence of a tag is `MonoOnly`, so items 1–19 keep
/// exactly the behaviour §36.5 shipped and every widening is an explicit,
/// reviewable edit in `COMPONENT_ARCHITECTURE.md`. There is deliberately no
/// flag, allow-list or environment variable that reaches this decision: §35.2
/// named the suppression class and §49.3 rejected reintroducing it.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
enum ItemScope {
    /// No `{scope: …}` tag. The item may account for mono movements only; a
    /// moved `truecolor` key citing it is refused.
    #[default]
    MonoOnly,
    /// `{scope: truecolor}` — the item may account for a moved `truecolor`
    /// key. Items 7, 11, 16 and 17 each anticipate truecolor movement and
    /// could not otherwise ever be discharged.
    TrueColor,
    /// `{scope: first-generation}` — the item covers the **first** recording
    /// of a key only. A *moved* key citing it is refused outright, which is
    /// what makes item 19's "may not be cited again for the same key"
    /// machine-checked rather than merely read.
    FirstGeneration,
    /// A `{scope: …}` tag whose value this guard does not know. Treated as
    /// `MonoOnly` for every refusal and additionally reported, so a typo
    /// widens nothing.
    Unrecognised,
}

/// Every item number declared in `COMPONENT_ARCHITECTURE.md` §20.10, with the
/// scope its row declares.
///
/// §20.10 is **five** tables — items 1–16, then 17, 18, 19 and 20 as separate
/// one-row tables. A parser that takes "the first table after the heading"
/// silently gets 1–16 and then rejects a correct citation of item 19, the item
/// that governs the first-generation digests. This scans the whole section for
/// table rows whose first cell is a number, so a further split costs nothing.
///
/// The scope tag is read from the whole row, not from a fixed cell, because
/// the tag belongs to the sentence that describes the change and the row's
/// cell boundaries are not stable.
fn visual_change_items(doc: &str) -> BTreeMap<u32, ItemScope> {
    let scope_re = Regex::new(r"\{\s*scope\s*:\s*([A-Za-z-]+)\s*\}").ok();
    let mut out = BTreeMap::new();
    let mut inside = false;
    for line in doc.lines() {
        if line.starts_with("### 20.10") {
            inside = true;
            continue;
        }
        if !inside {
            continue;
        }
        if line.starts_with("## ") || line.starts_with("### ") {
            break;
        }
        let trimmed = line.trim();
        if !trimmed.starts_with('|') {
            continue;
        }
        let cell = trimmed
            .trim_start_matches('|')
            .split('|')
            .next()
            .unwrap_or("")
            .trim();
        if let Ok(n) = cell.parse::<u32>() {
            let scope = scope_re
                .as_ref()
                .and_then(|re| re.captures(trimmed))
                .and_then(|c| c.get(1))
                .map_or(ItemScope::MonoOnly, |m| match m.as_str() {
                    "truecolor" => ItemScope::TrueColor,
                    "first-generation" => ItemScope::FirstGeneration,
                    _ => ItemScope::Unrecognised,
                });
            out.insert(n, scope);
        }
    }
    out
}

/// Every **declared** blocking marker in `COMPONENT_ARCHITECTURE.md`, as
/// `(1-based line number, the line)` (§49.4).
///
/// A marker is a section `**Status:` line that reads `BLOCKS the … bless`.
/// Only status lines are scanned: §49.4 itself quotes the marker in prose and
/// names its own pattern in backticks, and a scan of every line would match
/// those two and could never be discharged — a check that cannot go green is
/// as useless as one that cannot go red.
///
/// **What this proves, exactly.** It proves a blocker was *declared* and is
/// still standing. It does **not** prove that an undeclared blocker was
/// respected, and nothing here should be read as evidence of ordering: §36.5
/// is right that a committed tree is a state and not a history. A lane that
/// never writes the marker defeats this check completely; what the check buys
/// is that a blocker someone did write down cannot then be walked past
/// silently, because discharging it means editing one sentence in a
/// single-writer file, which is visible in a diff.
fn blocking_bless_markers(doc: &str) -> Result<Vec<(usize, String)>, String> {
    // `\bBLOCKS\b` is case-sensitive on purpose: "Unblocks the §39 re-bless"
    // is the *discharge* wording used by the same headers and must not match.
    // The `?` is why this returns a Result rather than an empty vector: a
    // refusal that degrades to "found nothing" when its own pattern fails to
    // build is a gate that cannot fail, which is the class this guard exists
    // inside.
    let marker = Regex::new(r"\bBLOCKS\b.*\bbless\b")
        .map_err(|e| format!("blocking-marker pattern does not compile: {e}"))?;
    let mut out = Vec::new();
    for (i, line) in doc.lines().enumerate() {
        let trimmed = line.trim();
        if !trimmed.trim_start_matches('*').starts_with("Status:") {
            continue;
        }
        if marker.is_match(&trimmed.replace('*', "")) {
            out.push((i.saturating_add(1), trimmed.to_owned()));
        }
    }
    Ok(out)
}

/// Expands `{a,b,c}` alternation groups into the full cross product, so one
/// matrix-generated key set is one ledger claim instead of 896 transcribed
/// lines. Whitespace inside a produced key is normalised to single spaces,
/// which is the form a baseline line uses.
fn expand_key_pattern(pattern: &str) -> Result<Vec<String>, String> {
    let mut out = vec![String::new()];
    let mut rest = pattern;
    loop {
        let Some(open) = rest.find('{') else {
            for s in &mut out {
                s.push_str(rest);
            }
            break;
        };
        let Some(close) = rest.get(open..).and_then(|r| r.find('}')).map(|i| open + i) else {
            return Err(format!("unbalanced `{{` in key pattern `{pattern}`"));
        };
        let head = rest.get(..open).unwrap_or_default();
        let alts: Vec<&str> = rest
            .get(open.saturating_add(1)..close)
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .collect();
        let mut next = Vec::new();
        for prefix in &out {
            for alt in &alts {
                let mut s = prefix.clone();
                s.push_str(head);
                s.push_str(alt);
                next.push(s);
            }
        }
        if next.len() > 100_000 {
            return Err(format!(
                "key pattern `{pattern}` expands past 100000 keys; write the keys out"
            ));
        }
        out = next;
        rest = rest.get(close.saturating_add(1)..).unwrap_or_default();
    }
    Ok(out
        .into_iter()
        .map(|s| s.split_whitespace().collect::<Vec<&str>>().join(" "))
        .filter(|s| !s.is_empty())
        .collect())
}

/// One classified entry of `docs/visual-changes.md`: a fenced block beneath a
/// `## Item <n>` heading. The entry-format block at the head of the ledger sits
/// **above** the first such heading and is therefore not an entry.
#[derive(Debug, Default)]
struct LedgerEntry {
    /// `Item 18 / 18b` — for messages only.
    label: String,
    /// The numbered §20.10 item this entry cites.
    cited: Option<u32>,
    class: Option<String>,
    /// The text left of the arrow on each `- moved:` claim line.
    moved: Vec<String>,
    moved_declared: Option<usize>,
    /// The expanded key claims of `- added:`.
    added: Vec<String>,
    added_declared: Option<usize>,
    /// Defects in the entry's own shape, reported only when it is engaged.
    defects: Vec<String>,
}

/// The `- name:` fields of one fenced entry block. A line that does not start a
/// field continues the previous one, which is how the ledger writes its
/// multi-line `- moved:` lists.
fn block_fields(block: &str) -> BTreeMap<String, String> {
    let mut out: BTreeMap<String, String> = BTreeMap::new();
    let mut current = String::new();
    for line in block.lines() {
        let mut started = false;
        if let Some(rest) = line.trim_start().strip_prefix("- ")
            && let Some((name, value)) = rest.split_once(':')
            && !name.is_empty()
            && name.chars().all(|c| c.is_ascii_lowercase() || c == '_')
        {
            current = name.to_owned();
            let field = out.entry(current.clone()).or_default();
            field.push_str(value.trim());
            field.push('\n');
            started = true;
        }
        if !started && !current.is_empty() {
            let field = out.entry(current.clone()).or_default();
            field.push_str(line.trim());
            field.push('\n');
        }
    }
    out
}

/// Whether a field's first line declares the empty set.
fn declares_none(field: &str) -> bool {
    field
        .lines()
        .next()
        .is_some_and(|l| l.trim_start().starts_with("none"))
}

/// The `N keys` / `N lines` count a field declares about itself, if any.
fn declared_count(field: &str) -> Option<usize> {
    let re = Regex::new(r"\b(\d+)\s+(?:keys?|lines?|entries|entry)\b").ok()?;
    re.captures(field)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse().ok())
}

/// The `<key> <old> → <new>` claim lines of a `- moved:` field: the text left
/// of the arrow, which is `<key> <old>` under the same "line minus its last
/// field" rule the writer uses. A line without an arrow is prose.
fn moved_claims(field: &str) -> Vec<String> {
    field
        .lines()
        .filter_map(|l| {
            let (left, right) = l.split_once('→').or_else(|| l.split_once("->"))?;
            let left = left.trim();
            if left.is_empty() || right.trim().is_empty() {
                return None;
            }
            Some(left.to_owned())
        })
        .collect()
}

/// The key claims of an `- added:` field: every backticked span, expanded
/// through `{a,b}` alternation. Backticks mark the claim so prose in the same
/// field cannot be mistaken for a key.
fn added_claims(field: &str) -> (Vec<String>, Vec<String>) {
    let mut keys = Vec::new();
    let mut defects = Vec::new();
    let mut rest = field;
    while let Some(open) = rest.find('`') {
        let after = rest.get(open.saturating_add(1)..).unwrap_or_default();
        let Some(end) = after.find('`') else { break };
        let span = after.get(..end).unwrap_or_default().trim();
        match expand_key_pattern(span) {
            Ok(expanded) => keys.extend(expanded),
            Err(e) => defects.push(e),
        }
        rest = after.get(end.saturating_add(1)..).unwrap_or_default();
    }
    (keys, defects)
}

/// Every entry of `docs/visual-changes.md`.
fn parse_ledger(ledger: &str) -> Vec<LedgerEntry> {
    let reason_re = Regex::new(r"§20\.10 item (\d+)").ok();
    let mut out = Vec::new();
    let mut item: Option<u32> = None;
    let mut sub = String::new();
    let mut in_fence = false;
    let mut block = String::new();
    for line in ledger.lines() {
        if line.trim_start().starts_with("```") {
            if in_fence {
                if let Some(n) = item {
                    let label = if sub.is_empty() {
                        format!("Item {n}")
                    } else {
                        format!("Item {n} / {sub}")
                    };
                    out.push(entry_from_block(&block, n, &label, reason_re.as_ref()));
                }
                block.clear();
            }
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            block.push_str(line);
            block.push('\n');
            continue;
        }
        if let Some(rest) = line.strip_prefix("## Item ") {
            let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
            item = digits.parse().ok();
            sub.clear();
        } else if line.starts_with("## ") {
            item = None;
            sub.clear();
        } else if let Some(rest) = line.strip_prefix("### ") {
            sub = rest.split('—').next().unwrap_or(rest).trim().to_owned();
        }
    }
    out
}

/// One entry, from its fenced block and the heading it sits beneath.
fn entry_from_block(
    block: &str,
    heading_item: u32,
    label: &str,
    reason_re: Option<&Regex>,
) -> LedgerEntry {
    let fields = block_fields(block);
    let empty = String::new();
    let moved_field = fields.get("moved").unwrap_or(&empty);
    let added_field = fields.get("added").unwrap_or(&empty);
    let reason = fields.get("reason").unwrap_or(&empty);
    let (added, mut defects) = added_claims(added_field);
    let moved = moved_claims(moved_field);
    let moved_none = declares_none(moved_field);
    let added_none = declares_none(added_field);
    if !moved_none && moved.is_empty() {
        defects.push(
            "`- moved:` is neither `none` nor one or more `<key> <old> → <new>` lines".to_owned(),
        );
    }
    if !added_none && added.is_empty() {
        defects
            .push("`- added:` is neither `none` nor one or more backticked key claims".to_owned());
    }
    let class = fields
        .get("class")
        .and_then(|f| f.split_whitespace().next())
        .map(str::to_ascii_lowercase);
    let cited = reason_re
        .and_then(|re| re.captures(reason))
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse().ok())
        .or(Some(heading_item));
    LedgerEntry {
        label: label.to_owned(),
        cited,
        class,
        moved,
        moved_declared: declared_count(moved_field),
        added,
        added_declared: declared_count(added_field),
        defects,
    }
}

/// Whether a `- moved:` claim accounts for `key`: the claim is `<key> <old>`,
/// so it either equals the key or is the key followed by its old value.
fn claim_covers(claim: &str, key: &str) -> bool {
    claim == key || claim.strip_prefix(key).is_some_and(|r| r.starts_with(' '))
}

/// The classification report. Pure: everything it needs arrives as strings.
///
/// **Ordering is not checked here and cannot be checked anywhere.** §36.5: a
/// committed tree is a state, not a history, so nothing in this guard proves
/// the classification was written before the bless. Do not read a green
/// `baseline_moves_are_classified` as evidence of the fixed order
/// change → capture → classify → bless. What it does prove is *completeness* —
/// the accounted key set equals the moved-and-added key set — plus the citation
/// and the two refusals.
fn report_baseline_moves(
    items: &BTreeMap<u32, ItemScope>,
    entries: &[LedgerEntry],
    moved: &[Movement],
    added: &[Addition],
) -> Result<(), String> {
    let mut problems: Vec<String> = Vec::new();
    if items.is_empty() {
        return Err(
            "COMPONENT_ARCHITECTURE.md §20.10 declares no numbered items; the citation check has \
             nothing to resolve against, which would let any citation pass"
                .to_owned(),
        );
    }

    // which entries account for which keys
    let mut engaged: Vec<bool> = vec![false; entries.len()];
    let mut unaccounted: Vec<String> = Vec::new();
    for m in moved {
        let mut covered = false;
        for (i, e) in entries.iter().enumerate() {
            if e.moved.iter().any(|c| claim_covers(c, &m.key)) {
                covered = true;
                if let Some(slot) = engaged.get_mut(i) {
                    *slot = true;
                }
            }
        }
        if !covered {
            unaccounted.push(format!(
                "  moved, unaccounted: {} :: {} ({} → {})",
                m.file, m.key, m.old, m.new
            ));
        }
    }
    for a in added {
        let mut covered = false;
        for (i, e) in entries.iter().enumerate() {
            if e.added.iter().any(|c| c == &a.key) {
                covered = true;
                if let Some(slot) = engaged.get_mut(i) {
                    *slot = true;
                }
            }
        }
        if !covered {
            unaccounted.push(format!("  added, unaccounted: {} :: {}", a.file, a.key));
        }
    }

    let moved_keys: BTreeSet<&str> = moved.iter().map(|m| m.key.as_str()).collect();
    let added_keys: BTreeSet<&str> = added.iter().map(|a| a.key.as_str()).collect();
    for (i, e) in entries.iter().enumerate() {
        if !engaged.get(i).copied().unwrap_or(false) {
            continue;
        }
        for d in &e.defects {
            problems.push(format!("  {}: {d}", e.label));
        }
        match e.cited {
            None => problems.push(format!("  {}: cites no numbered §20.10 item", e.label)),
            Some(n) if !items.contains_key(&n) => problems.push(format!(
                "  {}: cites §20.10 item {n}, which does not exist (§20.10 declares {:?})",
                e.label,
                items.keys().copied().collect::<Vec<u32>>()
            )),
            Some(n) if items.get(&n) == Some(&ItemScope::Unrecognised) => problems.push(format!(
                "  {}: cites §20.10 item {n}, whose row declares a `{{scope: …}}` tag that is \
                 neither `truecolor` nor `first-generation`. An unknown tag widens nothing and \
                 is reported rather than ignored (§49.3, fail-closed).",
                e.label
            )),
            Some(_) => {}
        }
        match e.class.as_deref() {
            Some("intended" | "fix") => {}
            Some("regression") => problems.push(format!(
                "  {}: `- class: regression` — a regression must be fixed, never blessed",
                e.label
            )),
            Some(other) => problems.push(format!(
                "  {}: `- class: {other}` is not one of intended | fix | regression",
                e.label
            )),
            None => problems.push(format!("  {}: has no `- class:` field", e.label)),
        }
        for c in &e.moved {
            if !moved_keys.iter().any(|k| claim_covers(c, k)) {
                problems.push(format!(
                    "  {}: `- moved:` claims `{c}`, which the diff did not move",
                    e.label
                ));
            }
        }
        for c in &e.added {
            if !added_keys.contains(c.as_str()) {
                problems.push(format!(
                    "  {}: `- added:` claims `{c}`, which the diff did not add",
                    e.label
                ));
            }
        }
        if let Some(n) = e.moved_declared
            && n != e.moved.len()
        {
            problems.push(format!(
                "  {}: `- moved:` declares {n} but lists {}",
                e.label,
                e.moved.len()
            ));
        }
        if let Some(n) = e.added_declared
            && n != e.added.len()
        {
            problems.push(format!(
                "  {}: `- added:` declares {n} but its key patterns expand to {}",
                e.label,
                e.added.len()
            ));
        }
    }

    if unaccounted.is_empty() && problems.is_empty() {
        return Ok(());
    }
    let mut msg = String::from(
        "docs/visual-changes.md does not account for this tree's baseline diff (§16.3, §20.10, \
         §36.5). Every moved and added key must be listed by an entry that cites a numbered \
         §20.10 item; co-presence of an entry is not enough.\n",
    );
    let shown = unaccounted.len().min(20);
    for line in unaccounted.iter().take(shown) {
        msg.push_str(line);
        msg.push('\n');
    }
    if unaccounted.len() > shown {
        msg.push_str(&format!(
            "  … and {} more unaccounted key(s)\n",
            unaccounted.len().saturating_sub(shown)
        ));
    }
    for line in &problems {
        msg.push_str(line);
        msg.push('\n');
    }
    Err(msg)
}

/// The whole guard over in-memory strings.
///
/// `files` is `(repository-relative path, base text, working text)` for every
/// non-frozen baseline; `frozen_changed` is the frozen evidence git reports as
/// touched, which is read from name-status rather than content because
/// `baseline/before/**` is binary.
fn evaluate_bless_guard(
    doc: &str,
    ledger: &str,
    files: &[(String, String, String)],
    frozen_changed: &[String],
) -> Result<(), String> {
    let mut refusals: Vec<String> = frozen_changed
        .iter()
        .map(|p| {
            format!(
                "  {p}: frozen pre-refactor evidence changed. Revert it (`git checkout -- {p}`); \
                 do not classify it."
            )
        })
        .collect();
    let mut moved: Vec<Movement> = Vec::new();
    let mut added: Vec<Addition> = Vec::new();
    let mut digest_files: BTreeSet<String> = BTreeSet::new();
    for (path, base, work) in files {
        let Some(kind) = classify_baseline(path) else {
            continue;
        };
        if kind == BaselineKind::Frozen {
            continue;
        }
        if kind == BaselineKind::Digest {
            digest_files.insert(path.clone());
        }
        let (m, a) = diff_baseline(path, kind, base, work);
        moved.extend(m);
        added.extend(a);
    }
    println!(
        "baseline_moves_are_classified: {} moved key(s), {} added key(s) across {} baseline file(s)",
        moved.len(),
        added.len(),
        files.len()
    );

    let items = visual_change_items(doc);
    let entries = parse_ledger(ledger);
    refusals.extend(refuse_while_a_bless_is_blocked(
        doc,
        &moved,
        &added,
        &digest_files,
    )?);
    refusals.extend(refuse_moved_digest_keys(
        &items,
        &entries,
        &moved,
        &digest_files,
    ));

    if !refusals.is_empty() {
        let mut msg = String::from("baseline changes refused outright (§36.5, §49.3, §49.4):\n");
        for r in &refusals {
            msg.push_str(r);
            msg.push('\n');
        }
        return Err(msg);
    }
    report_baseline_moves(&items, &entries, &moved, &added)
}

/// §49.4: refuse **any** digest addition or movement while
/// `COMPONENT_ARCHITECTURE.md` declares a live `BLOCKS the … bless` marker.
///
/// Coarse on purpose. A blocker on the bless is a blocker on the bless, so no
/// attempt is made to decide which keys a particular blocker covers — §36.5 is
/// right that ordering is not provable on a committed tree, and a check that
/// tried to scope the marker would be inventing the history it cannot read.
///
/// **What a green result proves, and only this: that no blocker was left
/// standing in the document.** It does not prove that an undeclared blocker
/// was respected, and it is not evidence about the order in which the tree
/// reached its state. Its whole force is that discharging it means editing one
/// sentence in a single-writer file, which a reviewer sees in the diff, rather
/// than walking past a marker silently — which is what happened in the
/// incident §49.4 records.
fn refuse_while_a_bless_is_blocked(
    doc: &str,
    moved: &[Movement],
    added: &[Addition],
    digest_files: &BTreeSet<String>,
) -> Result<Vec<String>, String> {
    let touched = moved.iter().any(|m| digest_files.contains(&m.file))
        || added.iter().any(|a| digest_files.contains(&a.file));
    if !touched {
        return Ok(Vec::new());
    }
    Ok(blocking_bless_markers(doc)?
        .into_iter()
        .map(|(line, text)| {
            // the status line carries a whole paragraph after the marker; quote
            // enough to identify the sentence and no more
            let quoted: String = text.chars().take(120).collect();
            let ellipsis = if text.chars().count() > 120 {
                "…"
            } else {
                ""
            };
            format!(
                "  COMPONENT_ARCHITECTURE.md:{line}: a blocking marker is still declared — \
                 \"{quoted}{ellipsis}\". No digest baseline key may be added or moved while it \
                 stands (§49.4). Discharge it by editing that `Status:` line in the same commit \
                 that lands the blocking change."
            )
        })
        .collect())
}

/// §49.3's two refusals over the moved digest keys.
///
/// 1. A moved `truecolor` key is refused **unless** an entry accounting for it
///    cites a §20.10 item whose row declares `{scope: truecolor}`. §36.5
///    justified the old unconditional refusal by §20.10's closing clause, but
///    that clause's predicate is *not on this list*, not *truecolor*; the two
///    coincide only while every numbered item is mono-only, and items 7, 11,
///    16 and 17 each anticipate truecolor movement. Refusing unconditionally
///    refuses those four items' own discharge, permanently.
/// 2. A moved key whose entry cites a `{scope: first-generation}` item is
///    refused outright, whatever its colour. That is item 19's "may not be
///    cited again for the same key", machine-checked instead of read.
///
/// Untagged items stay mono-only, so items 1–19 keep exactly today's
/// behaviour and widening one is an explicit edit in a single-writer file.
fn refuse_moved_digest_keys(
    items: &BTreeMap<u32, ItemScope>,
    entries: &[LedgerEntry],
    moved: &[Movement],
    digest_files: &BTreeSet<String>,
) -> Vec<String> {
    let mut out = Vec::new();
    for mv in moved {
        if !digest_files.contains(&mv.file) {
            continue;
        }
        let cited: Vec<(u32, ItemScope)> = entries
            .iter()
            .filter(|e| e.moved.iter().any(|c| claim_covers(c, &mv.key)))
            .filter_map(|e| e.cited)
            .map(|n| (n, items.get(&n).copied().unwrap_or_default()))
            .collect();
        for (n, scope) in &cited {
            if *scope == ItemScope::FirstGeneration {
                out.push(format!(
                    "  {}: `{}` MOVED ({} → {}) and the entry accounting for it cites §20.10 \
                     item {n}, whose row declares `{{scope: first-generation}}`. That item covers \
                     the FIRST recording of a key only and may not be cited again for the same \
                     key (§20.10 item 19, §49.2), so the movement is refused outright.",
                    mv.file, mv.key, mv.old, mv.new
                ));
            }
        }
        if mv.key.rsplit(' ').next() != Some("truecolor") {
            continue;
        }
        if cited.iter().any(|(_, s)| *s == ItemScope::TrueColor) {
            continue;
        }
        let who = if cited.is_empty() {
            "no ledger entry accounts for it".to_owned()
        } else {
            format!(
                "the entry accounting for it cites §20.10 item(s) {:?}, none of which declares \
                 that tag",
                cited.iter().map(|(n, _)| *n).collect::<Vec<u32>>()
            )
        };
        out.push(format!(
            "  {}: `{}` MOVED ({} → {}) and its colour field is `truecolor`. A moved truecolor \
             key is refused unless the entry accounting for it cites a §20.10 item whose row \
             declares `{{scope: truecolor}}`; {who}. An item with no `{{scope: …}}` tag is \
             mono-only, and §20.10's closing clause makes anything not on the list a regression \
             by construction (§49.3).",
            mv.file, mv.key, mv.old, mv.new
        ));
    }
    out
}

// ── the git and filesystem half ──

fn git(args: &[&str]) -> Result<std::process::Output, String> {
    Command::new("git")
        .args(args)
        .current_dir(root())
        .output()
        .map_err(|e| format!("git {}: {e}", args.join(" ")))
}

/// `git show <rev>:<path>`, or the empty string when the path did not exist.
fn git_show(rev: &str, path: &str) -> String {
    match git(&["show", &format!("{rev}:{path}")]) {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).into_owned(),
        _ => String::new(),
    }
}

fn resolve_rev(rev: &str, source: &str) -> Result<String, String> {
    let out = git(&[
        "rev-parse",
        "--verify",
        "--quiet",
        &format!("{rev}^{{commit}}"),
    ])?;
    if out.status.success() {
        return Ok(rev.to_owned());
    }
    Err(format!(
        "bless-guard base revision `{rev}` (from {source}) does not resolve. Falling back to HEAD \
         here would compare the tree with itself and pass vacuously, which is the failure this \
         gate exists to prevent, so the guard stops instead. In CI the checkout needs \
         `fetch-depth: 0`; locally set BLESS_GUARD_BASE to a revision that exists."
    ))
}

/// Resolve the guard base from the explicit CI variable, or from the pull
/// request base ref. Missing both is an error: falling back to `HEAD` would
/// compare the tree with itself and pass vacuously.
fn bless_guard_base_from(
    explicit: Option<&str>,
    github_base_ref: Option<&str>,
) -> Result<String, String> {
    if let Some(v) = explicit.filter(|v| !v.trim().is_empty()) {
        return resolve_rev(v.trim(), "BLESS_GUARD_BASE");
    }
    if let Some(v) = github_base_ref.filter(|v| !v.trim().is_empty()) {
        return resolve_rev(&format!("origin/{}", v.trim()), "GITHUB_BASE_REF");
    }
    Err(
        "bless-guard has no base revision. Set BLESS_GUARD_BASE explicitly (or provide GITHUB_BASE_REF on a pull request); comparing against HEAD is refused because it passes vacuously. CI checkouts must use `fetch-depth: 0`."
            .to_owned(),
    )
}

fn bless_guard_base() -> Result<String, String> {
    let explicit = std::env::var("BLESS_GUARD_BASE").ok();
    let github_base_ref = std::env::var("GITHUB_BASE_REF").ok();
    bless_guard_base_from(explicit.as_deref(), github_base_ref.as_deref())
}

/// `git diff -M --name-status <base>`: `(new path -> base path)` for renames,
/// and every baseline path the diff touches. `-M` is why Slice 5's move of the
/// app baselines into `apps/` does not fire hundreds of spurious entries.
fn diff_name_status(base: &str) -> Result<(BTreeMap<String, String>, BTreeSet<String>), String> {
    let out = git(&["diff", "-M", "--name-status", base, "--"])?;
    if !out.status.success() {
        return Err(format!(
            "git diff -M --name-status {base} failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    let mut renames = BTreeMap::new();
    let mut touched = BTreeSet::new();
    for line in text.lines() {
        let mut fields = line.split('\t');
        let Some(status) = fields.next() else {
            continue;
        };
        let Some(first) = fields.next() else { continue };
        match fields.next() {
            Some(second) if status.starts_with('R') || status.starts_with('C') => {
                renames.insert(second.to_owned(), first.to_owned());
                touched.insert(second.to_owned());
                touched.insert(first.to_owned());
            }
            _ => {
                touched.insert(first.to_owned());
            }
        }
    }
    Ok((renames, touched))
}

/// Untracked files, so a baseline that has never been committed is still seen.
fn untracked_files() -> Result<BTreeSet<String>, String> {
    let out = git(&["ls-files", "--others", "--exclude-standard"])?;
    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::to_owned)
        .collect())
}

/// Every baseline file on disk, by pattern rather than by name. A discovery
/// that stopped matching would find nothing to compare and the guard would exit
/// 0 forever; `the_baseline_inventory_is_not_empty` is the defence against that.
fn discover_baselines() -> BTreeSet<String> {
    WalkDir::new(root())
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            name != "target" && name != ".git"
        })
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .map(|e| rel(e.path()))
        .filter(|r| classify_baseline(r).is_some())
        .collect()
}

/// §16.3 as amended by §36, §20.10 and §36.5: every moved or added baseline key
/// is accounted for by a `docs/visual-changes.md` entry citing a numbered
/// §20.10 item, frozen evidence is never touched, a moved `truecolor` key needs
/// an item explicitly scoped for truecolor, and a first-generation item cannot
/// account for a moved key.
fn baseline_moves_are_classified() -> Result<(), String> {
    let base = bless_guard_base()?;
    let (renames, touched) = diff_name_status(&base)?;
    let untracked = untracked_files()?;
    let mut paths = discover_baselines();
    paths.extend(touched.iter().cloned());
    paths.extend(untracked.iter().cloned());
    paths.retain(|p| classify_baseline(p).is_some());

    let mut frozen_changed: Vec<String> = Vec::new();
    let mut files: Vec<(String, String, String)> = Vec::new();
    for path in &paths {
        if classify_baseline(path) == Some(BaselineKind::Frozen) {
            if touched.contains(path) || untracked.contains(path) {
                frozen_changed.push(path.clone());
            }
            continue;
        }
        let base_path = renames.get(path).cloned().unwrap_or_else(|| path.clone());
        files.push((
            path.clone(),
            git_show(&base, &base_path),
            read(&root().join(path)),
        ));
    }
    println!("baseline_moves_are_classified: base {base}");
    evaluate_bless_guard(
        &read(&root().join("COMPONENT_ARCHITECTURE.md")),
        &read(&root().join("docs/visual-changes.md")),
        &files,
        &frozen_changed,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doc_section_parser_reaches_the_authoritative_tail() {
        assert_eq!(
            top_level_doc_section_number("## §27 Adjudication"),
            Some(27)
        );
        assert_eq!(top_level_doc_section_number("## 68. Tail"), Some(68));
        assert_eq!(top_level_doc_section_number("### §27.1 Detail"), None);
        assert_eq!(top_level_doc_section_number("## Appendix A"), None);

        let scoped = doc_sections(
            "## 18. Migration\nskipped\n## §27 Adjudication\nkept\n### §27.1 Detail\nkept\n## 68. Tail\ntail\n",
        );
        assert!(!scoped.contains("skipped"), "§18 is outside the gate scope");
        assert!(scoped.contains("kept"), "§27 must be included");
        assert!(
            scoped.contains("tail"),
            "the open-ended tail must be included"
        );
    }

    #[test]
    fn doc_allow_list_rejects_stale_and_duplicate_entries() {
        let entries = parse_doc_allow("Live::member\nStale\nLive::member\n");
        let hits = BTreeMap::from([("Live::member".to_owned(), 1usize)]);
        let issues = stale_doc_allow_entries(&entries, &hits);
        assert!(
            issues
                .iter()
                .any(|issue| issue.contains("stale entry `Stale`")),
            "a satisfied allow-list must reject stale entries: {issues:?}"
        );
        assert!(
            issues
                .iter()
                .any(|issue| issue.contains("duplicate entry `Live::member`")),
            "a satisfied allow-list must reject duplicates: {issues:?}"
        );

        let live = parse_doc_allow("Live\n");
        assert!(
            stale_doc_allow_entries(&live, &hits).is_empty(),
            "a live entry covering an unresolved hit is the green case"
        );
    }

    #[test]
    fn required_scan_roots_reject_missing_and_empty_inputs() {
        let missing = scan_root_error(Path::new("crates/missing/src"), false, 0)
            .expect("a missing required root must fail");
        assert!(missing.contains("missing or not a directory"), "{missing}");

        let empty = scan_root_error(Path::new("crates/empty/src"), true, 0)
            .expect("an empty required root must fail");
        assert!(empty.contains("no Rust files"), "{empty}");

        assert!(
            scan_root_error(Path::new("crates/tui/src"), true, 1).is_none(),
            "a present root with Rust input is the green case"
        );
    }

    #[test]
    fn application_scan_rejects_a_due_app_without_apps_root() {
        let error = missing_apps_for_due(&[&APPS[0]])
            .expect("a due application must not pass with no apps root");
        assert!(error.contains("apps/ is missing"), "{error}");
        assert!(error.contains("apps/showcase/src"), "{error}");
        assert!(
            missing_apps_for_due(&[]).is_none(),
            "the accepted pre-Slice-5 state"
        );
    }

    #[test]
    fn application_scan_rejects_a_due_app_without_manifest() {
        let error = missing_application_manifest(&APPS[0]);
        assert!(error.contains("apps/showcase/Cargo.toml"), "{error}");
        assert!(error.contains("due application `showcase`"), "{error}");
    }

    #[test]
    fn binary_layout_gate_reports_wrong_owner_and_source() {
        let app = &APPS[0];
        let broken = binary_target_layout_hits(
            app,
            false,
            None,
            "wrong-package",
            Path::new("apps/showcase/src/other.rs"),
        );
        assert_eq!(
            broken.len(),
            2,
            "owner and path must both be checked: {broken:?}"
        );
        assert!(
            broken
                .iter()
                .any(|hit| hit.contains("wrong migration boundary"))
        );
        assert!(
            broken
                .iter()
                .any(|hit| hit.contains("apps/showcase/src/main.rs"))
        );
        assert!(
            binary_target_layout_hits(
                app,
                true,
                Some("junie-tui"),
                "junie-tui",
                &legacy_binary_source(app),
            )
            .is_empty()
        );
    }

    #[test]
    fn generic_component_copy_gate_reports_broken_fixture_and_allows_rain() {
        let broken = "\
fn render(&mut self) {}\n
let style = Style::new();\n
let block = Block::default();\n
buf.set_string(x, y, text, style);\n";
        let hits = generic_component_copy_hits("apps/showcase/src/page.rs", broken);
        assert_eq!(
            hits.len(),
            4,
            "every forbidden shape must be reported: {hits:?}"
        );
        assert!(hits.iter().any(|hit| hit.contains("fn render")), "{hits:?}");
        assert!(
            generic_component_copy_hits("apps/jackin-preview/src/rain.rs", broken).is_empty(),
            "rain.rs is the sole named exception"
        );
        let fixture = "#[cfg(test)]\nmod tests {\nfn render(&mut self) {}\n}\n";
        assert!(
            generic_component_copy_hits("apps/showcase/src/page.rs", fixture).is_empty(),
            "test-only helpers are not shipped application code"
        );
    }

    #[test]
    fn application_dispatch_gate_reports_broken_fixture_and_skips_tests() {
        let broken = "\
fn owns(x: Id) { registry.owns(x); }\n
fn locate(x: Id) { registry.locate(x); }\n
let _ = scrollbar::id_for(x);\n
let _ = parent.child(x);\n";
        let hits = owns_or_locate_hits("apps/showcase/src/page.rs", broken);
        assert_eq!(
            hits.len(),
            4,
            "every retired dispatch shape must be reported: {hits:?}"
        );
        let fixture = "#[cfg(test)]\nmod tests {\nfn route(x: Id) { registry.owns(x); }\n}\n";
        assert!(
            owns_or_locate_hits("apps/showcase/src/page.rs", fixture).is_empty(),
            "test-only helpers are not shipped application code"
        );
    }

    #[test]
    fn showcase_coverage_gate_reports_missing_public_component() {
        let cases = BTreeSet::from([
            "ProbeCase".to_owned(),
            "ButtonCase".to_owned(),
            "DialogCase".to_owned(),
        ]);
        let registry = BTreeSet::from(["Buttons".to_owned(), "Dialogs".to_owned()]);
        let pages = vec![
            (
                "apps/showcase/src/pages/buttons.rs".to_owned(),
                "trait Page {}
struct Demo;
impl Page for Demo {
    fn draw(&self) { Button::new(ID); }
    fn update(&mut self) { Button::new(ID); }
}"
                .to_owned(),
            ),
            (
                "apps/showcase/src/pages/dialogs.rs".to_owned(),
                "trait Page {}
struct DialogDemo;
impl Page for DialogDemo {
    fn draw(&self) { Dialog::confirm(ID); }
    fn update(&mut self) { Dialog::confirm(ID); }
}"
                .to_owned(),
            ),
        ];
        assert!(
            showcase_coverage_hits(&cases, &registry, &pages).is_empty(),
            "probe is test-only and the two public components are covered"
        );

        let missing = BTreeSet::from(["Buttons".to_owned()]);
        let hits = showcase_coverage_hits(&cases, &missing, &pages[..1]);
        assert_eq!(hits.len(), 1, "{hits:?}");
        assert!(hits[0].contains("DialogCase"), "{hits:?}");

        let empty = showcase_coverage_hits(&cases, &registry, &[]);
        assert!(
            empty
                .iter()
                .any(|hit| hit.contains("no registered showcase page")),
            "a missing page source must fail closed: {empty:?}"
        );

        let prose = vec![(
            "apps/showcase/src/pages/buttons.rs".to_owned(),
            "fn page() { let _label = \"Dialog\"; /* Button */ }".to_owned(),
        )];
        let prose_hits = showcase_coverage_hits(&cases, &registry, &prose);
        assert!(
            prose_hits.iter().any(|hit| hit.contains("ButtonCase"))
                && prose_hits.iter().any(|hit| hit.contains("DialogCase")),
            "comments and strings must not count as a component demonstration: {prose_hits:?}"
        );

        let imports = vec![(
            "apps/showcase/src/pages/buttons.rs".to_owned(),
            "use junie_tui::Button; fn page() {}".to_owned(),
        )];
        let import_hits = showcase_coverage_hits(&cases, &registry, &imports);
        assert!(
            import_hits.iter().any(|hit| hit.contains("ButtonCase")),
            "an import alone must not count as a component demonstration: {import_hits:?}"
        );

        let test_only = vec![(
            "apps/showcase/src/pages/buttons.rs".to_owned(),
            "trait Page {}
struct Demo;
impl Page for Demo {
    fn draw(&self) { Panel::new(ID); }
    fn update(&mut self) { Panel::new(ID); }
}
#[cfg(test)]
const BUTTON: Option<Button> = None;
#[cfg(test)]
impl Demo { fn draw(&self) { Dialog::new(ID); } }
"
            .to_owned(),
        )];
        let names = showcase_component_names(&test_only).expect("production roots parse");
        assert!(
            !names.contains("Button") && !names.contains("Dialog"),
            "test-only items must not count as showcase demonstrations: {names:?}"
        );
    }

    #[test]
    fn showcase_coverage_rejects_dead_helpers_and_signature_only_mentions() {
        let cases = BTreeSet::from(["ButtonCase".to_owned()]);
        let registry = BTreeSet::from(["Buttons".to_owned()]);
        let dead = vec![(
            "apps/showcase/src/pages/buttons.rs".to_owned(),
            "trait Page {}
struct Demo;
fn helper(_component: Button) {}
impl Page for Demo { fn draw(&self) {} fn update(&mut self) {} }
"
            .to_owned(),
        )];
        let hits = showcase_coverage_hits(&cases, &registry, &dead);
        assert!(
            hits.iter().any(|hit| hit.contains("ButtonCase")),
            "a dead helper/signature must not prove coverage: {hits:?}"
        );

        let reachable = vec![(
            "apps/showcase/src/pages/buttons.rs".to_owned(),
            "trait Page {}
struct Demo;
fn helper() { Button::new(ID); }
impl Page for Demo {
    fn draw(&self) { helper(); }
    fn update(&mut self) { helper(); }
}
"
            .to_owned(),
        )];
        assert!(
            showcase_coverage_hits(&cases, &registry, &reachable).is_empty(),
            "a helper reached from both production phases is real coverage"
        );
    }

    #[test]
    fn showcase_coverage_rejects_shared_demo_page_wrappers() {
        // This is the rejected fbaf994 shape: twenty-two one-line modules
        // expand a macro, while pages/mod.rs owns one generic DemoPage that
        // paints the whole component roster. The shared implementation must
        // not be able to impersonate twenty-two page owners.
        let variants = [
            "Overview",
            "Buttons",
            "Inputs",
            "TextAreas",
            "Forms",
            "Lists",
            "Trees",
            "Tables",
            "Editable",
            "Panels",
            "Sidebars",
            "Dialogs",
            "Progress",
            "Scrolling",
            "Terminal",
            "Editor",
            "Grid",
            "Chips",
            "Pickers",
            "Chrome",
            "Settings",
            "TaskRunner",
        ];
        let mut files = variants
            .iter()
            .map(|variant| {
                (
                    PathBuf::from(format!(
                        "apps/showcase/src/pages/{}.rs",
                        variant.to_ascii_lowercase()
                    )),
                    format!("super::define_page!({variant}Page, {variant});"),
                )
            })
            .collect::<Vec<_>>();
        files.push((
            PathBuf::from("apps/showcase/src/pages/mod.rs"),
            "trait Page {}
enum DemoKind { Buttons }
struct DemoPage { kind: DemoKind }
impl DemoPage {
    fn update_controls(&mut self) { Button::new(ID); Dialog::new(ID); }
    fn draw_content(&self) { Button::new(ID); Dialog::new(ID); }
}
impl Page for DemoPage {
    fn draw(&self) { self.draw_content(); }
    fn update(&mut self) { self.update_controls(); }
}
macro_rules! define_page {
    ($name:ident, $kind:ident) => {
        struct $name { inner: DemoPage }
        impl Page for $name {
            fn draw(&self) { self.inner.draw_content(); }
            fn update(&mut self) { self.inner.update_controls(); }
        }
    };
}
"
            .to_owned(),
        ));
        let registry = variants
            .iter()
            .map(|variant| (*variant).to_owned())
            .collect::<BTreeSet<_>>();
        let selected = showcase_page_sources(&files, &registry).expect("all 22 wrappers resolve");
        assert_eq!(selected.len(), SHOWCASE_PAGE_COUNT);
        assert!(
            selected
                .iter()
                .all(|(path, _)| !path.ends_with("/src/pages/mod.rs")),
            "the shared module cannot be a page source: {selected:?}"
        );

        let cases = BTreeSet::from(["ButtonCase".to_owned()]);
        let hits = showcase_coverage_hits(&cases, &registry, &selected);
        assert!(
            hits.iter()
                .any(|hit| hit.contains("every named page module must own")),
            "shared DemoPage roots must not satisfy per-page ownership: {hits:?}"
        );
        assert!(
            hits.iter().any(|hit| hit.contains("ButtonCase")),
            "components painted only by DemoPage must remain uncovered: {hits:?}"
        );
    }

    #[test]
    fn showcase_nav_registry_is_structural_and_non_empty() {
        let files = vec![(
            PathBuf::from("apps/showcase/src/app.rs"),
            "const NAV_ENTRIES: &[NavEntry] = &[NavEntry { id: PageId::Buttons }];".to_owned(),
        )];
        let (_, variants) = showcase_nav_registry(&files).expect("registry parses");
        assert_eq!(variants, BTreeSet::from(["Buttons".to_owned()]));

        let empty = vec![(
            PathBuf::from("apps/showcase/src/app.rs"),
            "const NAV_ENTRIES: &[NavEntry] = &[];".to_owned(),
        )];
        let error = showcase_nav_registry(&empty).expect_err("empty registry must fail closed");
        assert!(error.contains("no `PageId` entries"), "{error}");

        let test_only = vec![(
            PathBuf::from("apps/showcase/src/app.rs"),
            "#[cfg(test)]\nconst NAV_ENTRIES: &[NavEntry] = &[NavEntry { id: PageId::Buttons }];"
                .to_owned(),
        )];
        let error = showcase_nav_registry(&test_only)
            .expect_err("a test-only registry must not satisfy production coverage");
        assert!(error.contains("no `NAV_ENTRIES` registry"), "{error}");
    }

    fn page_contract_fixture(
        nav: &[&str],
        all: &[&str],
        dispatch: &[&str],
    ) -> Vec<(PathBuf, String)> {
        let variants: Vec<String> = (0..SHOWCASE_PAGE_COUNT).map(|i| format!("P{i}")).collect();
        let enum_body = variants.join(", ");
        let nav_body = nav
            .iter()
            .map(|variant| format!("NavEntry {{ id: PageId::{variant} }}"))
            .collect::<Vec<_>>()
            .join(", ");
        let all_body = all
            .iter()
            .map(|variant| format!("Self::{variant}"))
            .collect::<Vec<_>>()
            .join(", ");
        let dispatch_body = dispatch
            .iter()
            .map(|variant| format!("PageId::{variant} => ()"))
            .collect::<Vec<_>>()
            .join(", ");
        vec![(
            PathBuf::from("apps/showcase/src/app.rs"),
            format!(
                "enum PageId {{ {enum_body} }}
impl PageId {{ const ALL: [Self; 22] = [{all_body}]; }}
struct NavEntry;
const NAV_ENTRIES: &[NavEntry] = &[{nav_body}];
fn page(page: PageId) {{ match page {{ {dispatch_body} }} }}"
            ),
        )]
    }

    #[test]
    fn showcase_page_contract_rejects_short_duplicate_and_dead_axes() {
        let variants: Vec<String> = (0..SHOWCASE_PAGE_COUNT).map(|i| format!("P{i}")).collect();
        let refs: Vec<&str> = variants.iter().map(String::as_str).collect();
        let valid = showcase_page_contract(&page_contract_fixture(&refs, &refs, &refs));
        assert!(valid.is_ok(), "{valid:?}");

        let mut missing = refs.clone();
        missing.pop();
        let error = showcase_page_contract(&page_contract_fixture(&missing, &refs, &refs))
            .expect_err("a missing NAV page must fail closed");
        assert!(error.contains("NAV_ENTRIES has 21 page(s)"), "{error}");

        let mut duplicate = refs.clone();
        duplicate[21] = duplicate[0];
        let error = showcase_page_contract(&page_contract_fixture(&duplicate, &refs, &refs))
            .expect_err("a duplicate NAV page must fail closed");
        assert!(error.contains("NAV_ENTRIES contains duplicate"), "{error}");

        let empty = page_contract_fixture(&refs, &refs, &[]);
        let error = showcase_page_contract(&empty)
            .expect_err("a dead dispatch helper with no match arms must fail closed");
        assert!(error.contains("page dispatch has 0 page(s)"), "{error}");
    }

    fn signature_spec<'a>(
        component: &'a str,
        source: &'a str,
        state: Option<&'a str>,
        rects: usize,
    ) -> ClosureDrawSpec<'a> {
        ClosureDrawSpec {
            component,
            path: "isolated.rs",
            source,
            state,
            rects,
        }
    }

    #[test]
    fn closure_signature_gate_rejects_optional_and_nested_optional_results() {
        let panel = "impl Panel { pub fn draw<R>(&self, ui: &mut Ui<'_>, area: Rect, \
            body: impl FnOnce(&mut Ui<'_>, Rect) -> R) -> Option<R> { todo!() } }";
        let err = check_closure_draw_signature(&signature_spec("Panel", panel, None, 1))
            .expect_err("an optional Panel result must fail");
        assert!(err.contains("-> R`"), "{err}");

        let dialog = "impl Dialog { pub fn draw<R>(&self, ui: &mut Ui<'_>, area: Rect, \
            st: &DialogState, body: impl FnOnce(&mut Ui<'_>, Rect) -> R) \
            -> Option<Option<R>> { todo!() } }";
        let err =
            check_closure_draw_signature(&signature_spec("Dialog", dialog, Some("DialogState"), 1))
                .expect_err("a nested optional Dialog result must fail");
        assert!(err.contains("-> R`"), "{err}");
    }

    #[test]
    fn closure_signature_gate_rejects_tuple_geometry_and_wrong_slot_shapes() {
        let tuple_geometry = "impl SplitPane { pub fn draw<R>(&self, ui: &mut Ui<'_>, area: Rect, \
            st: &SplitPaneState) -> (Rect, Rect) { todo!() } }";
        check_closure_draw_signature(&signature_spec(
            "SplitPane",
            tuple_geometry,
            Some("SplitPaneState"),
            2,
        ))
        .expect_err("tuple-return geometry must fail");

        let tuple_slot = "impl SplitPane { pub fn draw<R>(&self, ui: &mut Ui<'_>, area: Rect, \
            st: &SplitPaneState, body: impl FnOnce(&mut Ui<'_>, (Rect, Rect)) -> R) -> R \
            { todo!() } }";
        check_closure_draw_signature(&signature_spec(
            "SplitPane",
            tuple_slot,
            Some("SplitPaneState"),
            2,
        ))
        .expect_err("a tuple argument is not the two-Rect slot");

        let two_rect_panel = "impl Panel { pub fn draw<R>(&self, ui: &mut Ui<'_>, area: Rect, \
            body: impl FnOnce(&mut Ui<'_>, Rect, Rect) -> R) -> R { todo!() } }";
        check_closure_draw_signature(&signature_spec("Panel", two_rect_panel, None, 1))
            .expect_err("Panel must have exactly one Rect body slot");
    }

    #[test]
    fn closure_signature_gate_accepts_current_component_sources() {
        closure_bearing_draw_signatures_are_exact()
            .expect("current Panel, Dialog and SplitPane signatures must pass");
    }

    const GRID_MODEL_GOOD: &str = "
pub struct CellRef<'a> { pub align: Option<Align>, marker: &'a str }
pub trait GridModel {
    fn row_count(&self) -> usize;
    fn row_key(&self, row: usize) -> ItemKey;
    fn cell(&self, row: usize, col: usize) -> Option<CellRef<'_>>;
    fn row_decor(&self, row: usize) -> RowDecor<'_> { todo!() }
    fn cell_decor(&self, row: usize, col: usize) -> CellDecor<'_> { todo!() }
    fn total(&self) -> RowTotal { todo!() }
    fn has_more(&self) -> bool { todo!() }
    fn read_only_reason(&self) -> Option<&str> { todo!() }
    fn actions(&self, row: usize, col: usize) -> &[CellAction] { todo!() }
}";

    #[test]
    fn grid_model_surface_gate_rejects_old_and_reintroduced_authorities() {
        let associated_row =
            GRID_MODEL_GOOD.replace("pub trait GridModel {", "pub trait GridModel { type Row;");
        let err = check_grid_model_surface(&associated_row, "isolated.rs")
            .expect_err("the old associated Row type must fail");
        assert!(err.contains("associated types"), "{err}");

        let col_count = GRID_MODEL_GOOD.replace(
            "fn row_count(&self) -> usize;",
            "fn row_count(&self) -> usize; fn col_count(&self) -> usize;",
        );
        let err = check_grid_model_surface(&col_count, "isolated.rs")
            .expect_err("a second column-count authority must fail");
        assert!(err.contains("col_count"), "{err}");
    }

    #[test]
    fn grid_model_surface_gate_rejects_non_structural_cells_and_align_sentinel() {
        let owned_cell = GRID_MODEL_GOOD.replace("Option<CellRef<'_>>", "CellRef<'_>");
        let err = check_grid_model_surface(&owned_cell, "isolated.rs")
            .expect_err("a non-optional cell cannot represent a ragged hole");
        assert!(err.contains("GridModel::cell"), "{err}");

        let align_sentinel = GRID_MODEL_GOOD.replace("Option<Align>", "Align");
        let err = check_grid_model_surface(&align_sentinel, "isolated.rs")
            .expect_err("Align::Left may not double as inheritance sentinel");
        assert!(err.contains("Option<Align>"), "{err}");
    }

    #[test]
    fn grid_model_surface_gate_accepts_fixture_and_current_source() {
        check_grid_model_surface(GRID_MODEL_GOOD, "isolated.rs")
            .expect("the exact isolated §61 surface must pass");
        grid_model_public_surface_is_exact().expect("current Grid surface must pass");
    }

    /// MA-2: the scan used to stop at the first `#[cfg(test)]`, so everything
    /// after a mid-file test helper was invisible to all forbidden-pattern
    /// rules. It must skip exactly the attributed item and carry on.
    #[test]
    fn non_test_lines_skips_only_the_cfg_test_item() {
        let src = "\
a();
#[cfg(test)]
fn helper() {
    let s = \"}\";
    inner();
}
b();
#[cfg(test)]
const K: u8 = 1;
c();
#[cfg(test)]
mod tests {
    fn t() {}
}
";
        let kept: Vec<&str> = non_test_lines(src).into_iter().map(|(_, l)| l).collect();
        assert_eq!(kept, vec!["a();", "b();", "c();"]);
        let lines: Vec<usize> = non_test_lines(src).into_iter().map(|(n, _)| n).collect();
        assert_eq!(lines, vec![1, 7, 10]);
    }

    #[test]
    fn legacy_forced_state_gate_rejects_production_and_skips_test_items() {
        let production =
            "impl Widget { pub fn state_override(self) {} }\nfn draw() { inherit_forced(); }";
        let hits = legacy_forced_state_hits("crates/tui/src/widget.rs", production);
        assert_eq!(hits.len(), 2);
        assert!(hits[0].contains("state_override"));
        assert!(hits[1].contains("inherit_forced"));

        let fixture =
            "#[cfg(test)]\nmod tests { fn probe() { state_override(); inherit_forced(); } }";
        assert!(legacy_forced_state_hits("crates/tui/src/widget.rs", fixture).is_empty());
    }

    #[test]
    fn reference_rendering_gate_fails_closed_on_nested_production_calls() {
        let production = "fn draw(ui: &mut Ui<'_>) { body(|_| ui.reference(None, |_| {})); }";
        let hits = ui_reference_hits("crates/tui/src/widget.rs", production);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].contains("ui.reference"));

        let fixture = "#[cfg(test)]\nfn probe(ui: &mut Ui<'_>) { ui.reference(None, |_| {}); }";
        assert!(ui_reference_hits("crates/tui/src/widget.rs", fixture).is_empty());
    }

    #[test]
    fn reference_rendering_gate_allows_only_fixture_and_application_paths() {
        let call = "fn draw(ui: &mut Ui<'_>) { ui.reference(None, |_| {}); }";
        for path in [
            "apps/showcase/src/page.rs",
            "crates/tui/examples/demo.rs",
            "crates/tui/tests/render.rs",
            "crates/tui-testing/src/conformance.rs",
        ] {
            assert!(ui_reference_hits(path, call).is_empty(), "{path}");
        }
        assert_eq!(
            ui_reference_hits("crates/consumer/src/lib.rs", call).len(),
            1
        );
    }

    // ── props are built once (§13, §16.5, §73) ──

    /// A screen that builds the **same** configured `Button` in both phases:
    /// the §13 defect written out. `draw` disables it, `update` does not, and
    /// nothing but this check can see the disagreement.
    const PROPS_BROKEN: &str = "\
const SAVE: Id = id!(\"save\");

impl App for Screen {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        Button::new(SAVE, \"Save\").variant(Variant::PRIMARY).update(cx).erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        Button::new(SAVE, \"Save\")
            .variant(Variant::PRIMARY)
            .disabled(true)
            .draw(ui, ui.full());
    }
}
";

    /// The same screen with the one private constructor §13 requires.
    const PROPS_FIXED: &str = "\
const SAVE: Id = id!(\"save\");

fn save_button(disabled: bool) -> Button<'static> {
    Button::new(SAVE, \"Save\").variant(Variant::PRIMARY).disabled(disabled)
}

impl App for Screen {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        save_button(self.busy).update(cx).erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        save_button(self.busy).draw(ui, ui.full());
    }
}
";

    /// Draw reaches the canonical constructor through a helper, while update
    /// hand-rolls the same ID. The unconfigured update call is intentional:
    /// once draw has a configured constructor, this is still two phase shapes
    /// for one component and must be rejected.
    const PROPS_HAND_ROLLED: &str = "\
const SAVE: Id = id!(\"save\");

fn save_button() -> Button<'static> {
    Button::new(SAVE, \"Save\").variant(Variant::PRIMARY)
}

impl App for Screen {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        Button::new(SAVE, \"Save\").update(cx).erase()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        save_button().draw(ui, ui.full());
    }
}
";

    /// Both phase roots reach the same constructor only through another
    /// helper, proving that the check is interprocedural rather than a count
    /// of direct calls in `App` methods.
    const PROPS_TRANSITIVE: &str = "\
const SAVE: Id = id!(\"save\");

fn save_button() -> Button<'static> {
    Button::new(SAVE, \"Save\").variant(Variant::PRIMARY)
}
fn update_button(cx: &mut Cx<'_>) { save_button().update(cx).erase(); }
fn draw_button(ui: &mut Ui<'_>) { save_button().draw(ui, ui.full()); }

impl App for Screen {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        update_button(cx)
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        draw_button(ui);
    }
}
";

    /// The red proof: the broken fixture is reported, the fixed one is not.
    /// A check nobody has ever seen fail is a check nobody has tested.
    #[test]
    fn props_built_once_gate_reports_the_two_phase_construction() {
        let hits = props_built_once_hits("apps/showcase/src/page.rs", PROPS_BROKEN);
        assert_eq!(hits.len(), 1, "{hits:?}");
        assert!(hits[0].contains("Button::new(SAVE)"), "{hits:?}");
        assert!(hits[0].contains("configured 2 times"), "{hits:?}");
        assert!(hits[0].contains("apps/showcase/src/page.rs"), "{hits:?}");

        assert!(
            props_built_once_hits("apps/showcase/src/page.rs", PROPS_FIXED).is_empty(),
            "the single-constructor form is the shape §13 asks for"
        );

        let hand_rolled = props_built_once_hits("apps/showcase/src/page.rs", PROPS_HAND_ROLLED);
        assert!(!hand_rolled.is_empty(), "{hand_rolled:?}");
        assert!(
            hand_rolled.iter().any(|hit| hit.contains("hand-rolled")),
            "{hand_rolled:?}"
        );
        assert!(
            props_built_once_hits("apps/showcase/src/page.rs", PROPS_TRANSITIVE).is_empty(),
            "both phases may reach the one constructor through helpers"
        );
    }

    /// The three shapes that are **not** the defect: an unconfigured
    /// `X::new(ID)` (which §13 exempts by name), a phase call (`update` is
    /// not configuration — the receiver after it is a `Response`), and a
    /// dynamic id, which is not the `const Id` this rule keys on.
    #[test]
    fn props_built_once_gate_keys_on_configuration_and_const_ids() {
        let unconfigured = "\
fn update(cx: &mut Cx<'_>) { Button::new(SAVE, \"Save\").update(cx); }
fn draw(ui: &mut Ui<'_>) { Button::new(SAVE, \"Save\").draw(ui, ui.full()); }
";
        assert!(props_built_once_hits("apps/a/src/s.rs", unconfigured).is_empty());

        let dynamic = "\
fn update(&self, cx: &mut Cx<'_>) { Button::new(self.id(), \"Save\").variant(V).update(cx); }
fn draw(&self, ui: &mut Ui<'_>) { Button::new(self.id(), \"Save\").variant(V).draw(ui, ui.full()); }
";
        assert!(props_built_once_hits("apps/a/src/s.rs", dynamic).is_empty());
    }

    /// Two different modules of one file each build their own screen's props
    /// once. Keying by file alone would report a violation that is not one.
    #[test]
    fn props_built_once_gate_keys_per_module() {
        let two_screens = "\
mod first {
    fn button() -> Button<'static> { Button::new(SAVE, \"Save\").variant(V) }
    fn update(cx: &mut Cx<'_>) { button().update(cx); }
    fn draw(ui: &mut Ui<'_>) { button().draw(ui, ui.full()); }
}
mod second {
    fn button() -> Button<'static> { Button::new(SAVE, \"Save\").variant(V) }
    fn update(cx: &mut Cx<'_>) { button().update(cx); }
    fn draw(ui: &mut Ui<'_>) { button().draw(ui, ui.full()); }
}
";
        let hits = props_built_once_hits("apps/a/src/s.rs", two_screens);
        assert!(hits.is_empty(), "{hits:?}");
    }

    /// A test module builds the same configured props many times on purpose —
    /// that is the fixture, not the screen (MA-2's rule, applied here).
    #[test]
    fn props_built_once_gate_skips_test_items() {
        let source = format!("#[cfg(test)]\nmod tests {{\n{PROPS_BROKEN}\n}}\n");
        assert!(props_built_once_hits("apps/a/src/s.rs", &source).is_empty());
    }

    /// The fail-closed half: a scope that observed nothing cannot report `ok`.
    #[test]
    fn props_built_once_gate_fails_closed_on_an_empty_scan() {
        assert!(props_vacuity_hits(&[("crates/tui/examples", 14)], 36).is_empty());

        let no_files = props_vacuity_hits(&[("apps", 0), ("crates/tui/examples", 14)], 36);
        assert_eq!(no_files.len(), 1, "{no_files:?}");
        assert!(no_files[0].contains("apps"), "{no_files:?}");

        let nothing_seen = props_vacuity_hits(&[("crates/tui/examples", 14)], 0);
        assert_eq!(nothing_seen.len(), 1, "{nothing_seen:?}");
        assert!(
            nothing_seen[0].contains("looking at nothing"),
            "{nothing_seen:?}"
        );
    }

    #[test]
    fn cache_type_gate_ignores_test_only_cache_payloads() {
        let source = "\
fn production(ui: &mut Ui<'_>) { ui.cache::<Layout>(ID); }
#[cfg(test)]
mod tests {
    fn cache_namespace_probe(ui: &mut Ui<'_>) { ui.cache::<u32>(ID); }
}
";
        assert_eq!(
            production_cache_types(source).expect("valid cache scan"),
            BTreeSet::from(["Layout".to_owned()])
        );
    }

    // ── bless-guard (§16.3, §20.10, §36.5) ──

    /// A §20.10 stand-in that reproduces the real document's **five** tables.
    const DOC: &str = "\
### 20.10 Intentional visual changes

| # | Change | Why | How it is reviewed |
|---|---|---|---|
| 1 | **Mono legibility fallbacks** | … | … |
| 2 | **Layer compositing order** | … | … |

| 17 | **`Anchor::Point` flips** | … | … |


| 18 | **Mono `DISABLED` gains `DIM`** | … | … |

| 19 | **First-generation digests**; may not be cited again for the same key. \
`{scope: first-generation}` | … | … |

| 20 | **Forcing stops erasing the props half.** `{scope: truecolor}` | … | … |

## Appendix A — Slice plan
";

    /// The same §20.10, in a document that still declares a live blocking
    /// marker on its own status line — the shape §49.4 records the bless
    /// commit's tree as having had.
    const DOC_BLOCKED: &str = "\
## §39 Adjudication — forcing substitutes for the runtime

**Status: accepted. BLOCKS the §36 first-generation bless — see §39.4.** Fresh adjudication.

### 20.10 Intentional visual changes

| # | Change | Why | How it is reviewed |
|---|---|---|---|
| 1 | **Mono legibility fallbacks** | … | … |

## Appendix A — Slice plan
";

    const BASE_BASELINE: &str = "\
# digest baseline: name w h theme color hash
render::components::tabs::pressed 120 40 junie mono 5517de00b23ac747
render::components::tabs::pressed 120 40 junie truecolor aaaaaaaaaaaaaaaa
";

    const WORK_BASELINE: &str = "\
# digest baseline: name w h theme color hash
render::components::tabs::pressed 120 40 junie mono 8531aef99ed82a7c
render::components::tabs::pressed 120 40 junie truecolor aaaaaaaaaaaaaaaa
";

    const LEDGER_WITHOUT_ENTRY: &str = "\
# Visual changes ledger

## Item 1 — Mono legibility fallbacks

captures / classification: `(pending — filled when the change lands)`
";

    const LEDGER_WITH_ENTRY: &str = "\
# Visual changes ledger

## Item 1 — Mono legibility fallbacks

### 1a — `Tabs` paints §11.4's mono `PRESSED` bracket

```
- surface:   junie-tui/tabs/pressed @ 120x40 / junie / mono
- captures:  none under `shots/` — headless `Scene` matrix; frame-text dump attached
- tests:     crates/tui/tests/baselines/components.txt
- moved:     1 line, every one `mono`:
  render::components::tabs::pressed 120 40 junie mono 5517de00b23ac747 → 8531aef99ed82a7c
- added:     none
- class:     fix
- reason:    §20.10 item 1 (mono legibility fallbacks).
```
";

    /// An entry that cites the `{scope: truecolor}` item and claims both the
    /// mono and the truecolor movement of the same key.
    const LEDGER_ITEM_20_TRUECOLOR: &str = "\
# Visual changes ledger

## Item 20 — forcing stops erasing the props half

```
- surface:   junie-tui/tabs/pressed @ 120x40 / junie
- captures:  none under `shots/` — headless `Scene` matrix; frame-text dump attached
- tests:     crates/tui/tests/baselines/components.txt
- moved:     2 lines:
  render::components::tabs::pressed 120 40 junie mono 5517de00b23ac747 → 8531aef99ed82a7c
  render::components::tabs::pressed 120 40 junie truecolor aaaaaaaaaaaaaaaa → bbbbbbbbbbbbbbbb
- added:     none
- class:     fix
- reason:    §20.10 item 20 (forcing stops erasing the props half).
```
";

    /// An entry that cites the `{scope: first-generation}` item for a key the
    /// diff **moved** — the second citation item 19 forbids.
    const LEDGER_ITEM_19_FIRST_GEN: &str = "\
# Visual changes ledger

## Item 19 — first-generation component digests

```
- surface:   junie-tui/tabs/pressed @ 120x40 / junie / mono
- captures:  none under `shots/` — headless `Scene` matrix; frame-text dump attached
- tests:     crates/tui/tests/baselines/components.txt
- moved:     1 line:
  render::components::tabs::pressed 120 40 junie mono 5517de00b23ac747 → 8531aef99ed82a7c
- added:     none
- class:     intended
- reason:    §20.10 item 19 (first-generation digests).
```
";

    fn one_file(base: &str, work: &str) -> Vec<(String, String, String)> {
        vec![(
            "crates/tui/tests/baselines/components.txt".to_owned(),
            base.to_owned(),
            work.to_owned(),
        )]
    }

    /// The red half of the COORDINATION.md demonstration: a moved key with no
    /// ledger entry accounting for it must fail. A guard hard-wired to `Ok(())`
    /// fails this test.
    #[test]
    fn a_moved_baseline_without_a_ledger_entry_fails() {
        let err = evaluate_bless_guard(
            DOC,
            LEDGER_WITHOUT_ENTRY,
            &one_file(BASE_BASELINE, WORK_BASELINE),
            &[],
        )
        .expect_err("an unclassified move must fail");
        assert!(
            err.contains("render::components::tabs::pressed 120 40 junie mono"),
            "the failure must name the key: {err}"
        );
    }

    /// The green half: the same input with the entry present passes. A guard
    /// hard-wired to `Err` fails this test.
    #[test]
    fn the_same_move_with_a_ledger_entry_passes() {
        evaluate_bless_guard(
            DOC,
            LEDGER_WITH_ENTRY,
            &one_file(BASE_BASELINE, WORK_BASELINE),
            &[],
        )
        .expect("the same move, classified, must pass");
    }

    #[test]
    fn an_unchanged_tree_reports_no_moves() {
        let (moved, added) = diff_baseline(
            "crates/tui/tests/baselines/components.txt",
            BaselineKind::Digest,
            BASE_BASELINE,
            BASE_BASELINE,
        );
        assert!(moved.is_empty() && added.is_empty(), "{moved:?} {added:?}");
        evaluate_bless_guard(
            DOC,
            LEDGER_WITHOUT_ENTRY,
            &one_file(BASE_BASELINE, BASE_BASELINE),
            &[],
        )
        .expect("an unchanged tree has nothing to classify");
    }

    /// §20.10 is split across tables. A parser that takes only the first table
    /// loses later rows; an accidental numbering gap makes a citation resolve
    /// to no visual-change authority at all.
    #[test]
    fn the_2010_item_list_survives_the_split_tables() {
        let doc = read(&root().join("COMPONENT_ARCHITECTURE.md"));
        let items = visual_change_items(&doc);
        let want: BTreeSet<u32> = (1..=31).collect();
        assert_eq!(
            items.keys().copied().collect::<BTreeSet<u32>>(),
            want,
            "§20.10 item numbers"
        );
    }

    /// §49.3's two scope tags must actually be present on the rows the guard
    /// resolves them from, or the refusals it derives from them are decoration.
    ///
    /// Item 19 is the first-generation item: without `{scope: first-generation}`
    /// its "may not be cited again for the same key" clause is read by people
    /// and enforced by nothing. Item 20 is the first `{scope: truecolor}` item:
    /// without the tag it cannot discharge its own truecolor movement.
    #[test]
    fn the_2010_scope_tags_are_declared_where_the_guard_needs_them() {
        let doc = read(&root().join("COMPONENT_ARCHITECTURE.md"));
        let items = visual_change_items(&doc);
        assert_eq!(
            items.get(&19),
            Some(&ItemScope::FirstGeneration),
            "§20.10 item 19's row must end its `Change` cell with the literal text \
             `{{scope: first-generation}}` (COMPONENT_ARCHITECTURE.md is single-writer, so this \
             is Lane A's edit). Without it, item 19 is treated as mono-only and a second \
             movement of a first-generation key citing item 19 is refused only when the key is \
             truecolor."
        );
        assert_eq!(
            items.get(&20),
            Some(&ItemScope::TrueColor),
            "§20.10 item 20's row must carry the literal text `{{scope: truecolor}}`"
        );
        assert_eq!(items.get(&21), Some(&ItemScope::MonoOnly));
        assert_eq!(items.get(&22), Some(&ItemScope::FirstGeneration));
        assert_eq!(items.get(&23), Some(&ItemScope::TrueColor));
        assert_eq!(items.get(&24), Some(&ItemScope::FirstGeneration));
        assert_eq!(items.get(&25), Some(&ItemScope::FirstGeneration));
        assert_eq!(items.get(&26), Some(&ItemScope::FirstGeneration));
        assert_eq!(items.get(&27), Some(&ItemScope::FirstGeneration));
        assert_eq!(items.get(&28), Some(&ItemScope::TrueColor));
    }

    /// The tag is read from the row text, wherever in the row it sits, and an
    /// untagged row is mono-only. This is the green half of
    /// `the_2010_scope_tags_are_declared_where_the_guard_needs_them`, proven
    /// on a fixture because `COMPONENT_ARCHITECTURE.md` is single-writer.
    #[test]
    fn the_scope_tag_is_read_from_the_row_text() {
        let items = visual_change_items(DOC);
        assert_eq!(items.get(&1), Some(&ItemScope::MonoOnly), "untagged");
        assert_eq!(items.get(&18), Some(&ItemScope::MonoOnly), "untagged");
        assert_eq!(items.get(&19), Some(&ItemScope::FirstGeneration));
        assert_eq!(items.get(&20), Some(&ItemScope::TrueColor));
    }

    /// An unrecognised tag must widen nothing and must be reported, not
    /// silently read as "untagged".
    #[test]
    fn an_unrecognised_scope_tag_is_mono_only_and_reported() {
        let doc = DOC.replace("{scope: truecolor}", "{scope: everything}");
        assert_eq!(
            visual_change_items(&doc).get(&20),
            Some(&ItemScope::Unrecognised)
        );
        // it does not widen: the truecolor movement is still refused
        let work = WORK_BASELINE.replace("aaaaaaaaaaaaaaaa", "bbbbbbbbbbbbbbbb");
        let err = evaluate_bless_guard(
            &doc,
            LEDGER_ITEM_20_TRUECOLOR,
            &one_file(BASE_BASELINE, &work),
            &[],
        )
        .expect_err("an unknown tag may not discharge the truecolor refusal");
        assert!(
            err.contains("refused unless the entry accounting for it cites"),
            "{err}"
        );
        // and it is named, rather than being read as an untagged row
        let mono_only = LEDGER_ITEM_20_TRUECOLOR
            .replace(
                "  render::components::tabs::pressed 120 40 junie truecolor aaaaaaaaaaaaaaaa →                  bbbbbbbbbbbbbbbb\n",
                "",
            )
            .replace("2 lines", "1 line");
        let err = evaluate_bless_guard(
            &doc,
            &mono_only,
            &one_file(BASE_BASELINE, WORK_BASELINE),
            &[],
        )
        .expect_err("an unknown tag is reported");
        assert!(
            err.contains("neither `truecolor` nor `first-generation`"),
            "{err}"
        );
    }

    /// §49.3, half one: a `{scope: truecolor}` item discharges its own
    /// truecolor movement. Without this the four already-numbered items that
    /// anticipate truecolor movement — 7, 11, 16 and 17 — could never be
    /// discharged at all.
    #[test]
    fn a_moved_truecolor_key_is_allowed_by_an_item_tagged_truecolor() {
        let work = WORK_BASELINE.replace("aaaaaaaaaaaaaaaa", "bbbbbbbbbbbbbbbb");
        evaluate_bless_guard(
            DOC,
            LEDGER_ITEM_20_TRUECOLOR,
            &one_file(BASE_BASELINE, &work),
            &[],
        )
        .expect("an item declaring `{scope: truecolor}` accounts for its own truecolor movement");
    }

    /// §49.3, half two: item 19's "may not be cited again for the same key",
    /// machine-checked. The key here is `mono`, so nothing but the
    /// first-generation tag can refuse it.
    #[test]
    fn a_moved_key_citing_a_first_generation_item_is_refused() {
        let err = evaluate_bless_guard(
            DOC,
            LEDGER_ITEM_19_FIRST_GEN,
            &one_file(BASE_BASELINE, WORK_BASELINE),
            &[],
        )
        .expect_err("a first-generation item may not account for a movement");
        assert!(err.contains("first-generation"), "{err}");
        assert!(err.contains("item 19"), "{err}");
    }

    /// §49.4: a declared blocker refuses every digest change while it stands.
    #[test]
    fn a_live_blocking_marker_refuses_every_digest_change() {
        let err = evaluate_bless_guard(
            DOC_BLOCKED,
            LEDGER_WITH_ENTRY,
            &one_file(BASE_BASELINE, WORK_BASELINE),
            &[],
        )
        .expect_err("a declared blocker refuses the bless");
        assert!(err.contains("blocking marker is still declared"), "{err}");
        assert!(
            err.contains("BLOCKS the §36 first-generation bless"),
            "{err}"
        );
    }

    /// …and stops refusing the moment that sentence is edited. Discharge is an
    /// edit to one line of a single-writer file, visible in the diff.
    #[test]
    fn a_discharged_blocking_marker_stops_refusing() {
        let doc = DOC_BLOCKED.replace(
            "BLOCKS the §36 first-generation bless — see §39.4.",
            "Landed; the §36 first-generation bless is unblocked (§49).",
        );
        assert!(
            blocking_bless_markers(&doc)
                .expect("pattern compiles")
                .is_empty(),
            "the discharged wording must not match"
        );
        evaluate_bless_guard(
            &doc,
            LEDGER_WITH_ENTRY,
            &one_file(BASE_BASELINE, WORK_BASELINE),
            &[],
        )
        .expect("a discharged marker refuses nothing");
    }

    /// The refusal is on the *change*, not on the marker: a tree that moves no
    /// digest key passes with the marker standing, so the check cannot be
    /// mistaken for a blanket veto on committing while a blocker exists.
    #[test]
    fn a_blocking_marker_alone_refuses_nothing() {
        evaluate_bless_guard(
            DOC_BLOCKED,
            LEDGER_WITHOUT_ENTRY,
            &one_file(BASE_BASELINE, BASE_BASELINE),
            &[],
        )
        .expect("no digest key moved or was added");
    }

    /// §49.6: the guard's base falls back to `HEAD` when nothing sets one, and
    /// CI runs on **push to `main`** as well as on pull requests. On the push
    /// leg there is no `GITHUB_BASE_REF`, so without an explicit base a clean
    /// checkout is diffed against itself and the guard reports `0 moved,
    /// 0 added` on every direct commit — which is what it did for this whole
    /// session. The push leg must therefore name a base explicitly.
    #[test]
    fn the_ci_push_leg_gives_the_bless_guard_a_base() {
        let ci = read(&root().join(".github/workflows/ci.yml"));
        assert!(
            ci.contains("  push:"),
            "the workflow no longer has a push leg; this check is about that leg"
        );
        let lines: Vec<&str> = ci.lines().collect();
        // the `run:` step, not the gate→requirement comment block at the head
        // of the file, which names the same command
        let run = lines
            .iter()
            .position(|l| {
                !l.trim_start().starts_with('#')
                    && l.contains("run: cargo run -p xtask -- bless-guard")
            })
            .expect("ci.yml has a step that runs the bless guard");
        let start = run.saturating_sub(8);
        let window = lines.get(start..run).unwrap_or_default().join("\n");
        assert!(
            window.contains("BLESS_GUARD_BASE:"),
            "the bless-guard step must set BLESS_GUARD_BASE; without it the push leg diffs \
             the tree against itself. Step context:\n{window}"
        );
        assert!(
            window.contains("github.event.before"),
            "BLESS_GUARD_BASE on the push leg must be `${{{{ github.event.before }}}}` — the \
             commit the push moved `main` off. Step context:\n{window}"
        );
    }

    #[test]
    fn missing_guard_base_is_refused_instead_of_defaulting_to_head() {
        let err = bless_guard_base_from(None, None)
            .expect_err("a guard without an explicit or pull-request base must stop");
        assert!(err.contains("no base revision"), "{err}");
        assert!(err.contains("BLESS_GUARD_BASE"), "{err}");
    }

    #[test]
    fn invalid_explicit_guard_base_does_not_fall_back_to_pull_request_base() {
        let err = bless_guard_base_from(Some("not-a-real-revision"), Some("main"))
            .expect_err("an invalid explicit base must stop");
        assert!(err.contains("not-a-real-revision"), "{err}");
        assert!(err.contains("does not resolve"), "{err}");
    }

    /// The defence against the third vacuous-pass mode: discovery patterns that
    /// stop matching find nothing to compare and the guard exits 0 forever.
    #[test]
    fn the_baseline_inventory_is_not_empty() {
        let found = discover_baselines();
        for want in [
            "crates/tui/tests/baselines/components.txt",
            "crates/tui/tests/perf_baseline.txt",
        ] {
            assert!(found.contains(want), "{want} not discovered: {found:?}");
        }
    }

    /// A moved `truecolor` key whose entry cites an **untagged** item is
    /// refused, even when the ledger otherwise accounts for it. No item before
    /// 20 carries `{scope: truecolor}`, so this is §36.5's behaviour unchanged.
    ///
    /// The assertion names the refusal sentence, not merely the word
    /// `truecolor`: with the refusal removed this input still fails the
    /// *completeness* check, so a weaker assertion would pass on a guard that
    /// had stopped refusing anything.
    #[test]
    fn a_moved_truecolor_key_is_refused_outright() {
        let work = WORK_BASELINE.replace("aaaaaaaaaaaaaaaa", "bbbbbbbbbbbbbbbb");
        let err =
            evaluate_bless_guard(DOC, LEDGER_WITH_ENTRY, &one_file(BASE_BASELINE, &work), &[])
                .expect_err("a truecolor movement is a regression by construction");
        assert!(
            err.contains("refused unless the entry accounting for it cites"),
            "{err}"
        );
    }

    /// `regression` is the ledger's own "must be fixed, never blessed".
    #[test]
    fn a_regression_class_fails_the_guard() {
        let ledger = LEDGER_WITH_ENTRY.replace("- class:     fix", "- class:     regression");
        let err = evaluate_bless_guard(DOC, &ledger, &one_file(BASE_BASELINE, WORK_BASELINE), &[])
            .expect_err("a regression may not be blessed");
        assert!(err.contains("regression"), "{err}");
    }

    /// A citation of an item §20.10 does not declare is not a citation.
    #[test]
    fn a_citation_of_a_nonexistent_2010_item_fails() {
        let ledger = LEDGER_WITH_ENTRY.replace("§20.10 item 1 ", "§20.10 item 42 ");
        let err = evaluate_bless_guard(DOC, &ledger, &one_file(BASE_BASELINE, WORK_BASELINE), &[])
            .expect_err("item 42 does not exist");
        assert!(err.contains("item 42"), "{err}");
    }

    #[test]
    fn frozen_evidence_fails_without_classification() {
        let err = evaluate_bless_guard(
            DOC,
            LEDGER_WITH_ENTRY,
            &[],
            &["tests/baselines/tablepro.txt".to_owned()],
        )
        .expect_err("frozen evidence may not change");
        assert!(err.contains("Revert it"), "{err}");
    }

    /// The perf rule: the `ns` column is re-measured per machine, so a timing
    /// difference is not a movement — but an allocation or hit-count one is.
    #[test]
    fn perf_baselines_ignore_the_ns_column_and_see_the_rest() {
        let base = "frame_showcase_lists_120x40 8275 0 0 9 7\n";
        let noise = "frame_showcase_lists_120x40 9111 0 0 9 7\n";
        let real = "frame_showcase_lists_120x40 8275 0 0 10 7\n";
        let (moved, _) =
            diff_baseline("a/tests/perf_baseline.txt", BaselineKind::Perf, base, noise);
        assert!(
            moved.is_empty(),
            "timing noise is not a movement: {moved:?}"
        );
        let (moved, _) = diff_baseline("a/tests/perf_baseline.txt", BaselineKind::Perf, base, real);
        assert_eq!(moved.len(), 1, "a hit-count change is a movement");
    }

    /// Frozen paths win over the digest and perf patterns.
    #[test]
    fn baseline_paths_are_classified_by_pattern() {
        use BaselineKind::{Digest, Frozen, Perf};
        for (path, want) in [
            ("crates/tui/tests/baselines/components.txt", Some(Digest)),
            ("apps/showcase/tests/baselines/showcase.txt", Some(Digest)),
            ("crates/tui/tests/perf_baseline.txt", Some(Perf)),
            ("tests/perf_baseline.txt", Some(Frozen)),
            ("tests/showcase_baseline.txt", Some(Frozen)),
            ("tests/baselines/tablepro.txt", Some(Frozen)),
            (
                "baseline/before/showcase_forms_default_120x40.png",
                Some(Frozen),
            ),
            ("crates/tui/src/lib.rs", None),
        ] {
            assert_eq!(classify_baseline(path), want, "{path}");
        }
    }

    /// One matrix-generated set is one claim; the declared count must match.
    #[test]
    fn a_key_pattern_expands_to_the_matrix_it_names() {
        let keys = expand_key_pattern(
            "render::components::{meter,brand}::{default,focused} {120 40,40 10} {junie} {mono}",
        )
        .expect("expands");
        assert_eq!(keys.len(), 8);
        assert!(keys.contains(&"render::components::meter::default 120 40 junie mono".to_owned()));
    }

    #[test]
    fn capture_matrix_covers_every_declared_cell() {
        let expected_sizes = [(80, 24), (100, 30), (120, 40), (160, 50)];
        let expected_colors = ["truecolor", "256", "16", "mono"];
        let expected_themes = ["junie", "paper"];
        let expected_apps = [
            CaptureApp {
                name: "showcase",
                binary: "showcase",
                extra_args: "",
            },
            CaptureApp {
                name: "tablepro",
                binary: "tablepro",
                extra_args: "",
            },
            CaptureApp {
                name: "jackin-preview",
                binary: "jackin-preview",
                extra_args: "--motion paused --frame 0",
            },
        ];
        let expected: BTreeSet<CaptureCase> = expected_sizes
            .into_iter()
            .flat_map(|(width, height)| {
                expected_colors.into_iter().flat_map(move |color| {
                    expected_themes.into_iter().flat_map(move |theme| {
                        expected_apps.into_iter().map(move |app| CaptureCase {
                            app,
                            width,
                            height,
                            color,
                            theme,
                        })
                    })
                })
            })
            .collect();
        let actual: BTreeSet<CaptureCase> = capture_matrix_cases().into_iter().collect();
        assert_eq!(actual, expected);
        assert_eq!(actual.len(), 3 * 4 * 4 * 2);
    }

    #[test]
    fn capture_contract_rejects_axis_and_script_shortcuts() {
        assert!(capture_axes_contract_hits().is_empty());
        let missing_png = "BIN=ok\nCOLOR=${COLOR:-truecolor}\ncase \"$COLOR\" in\ntruecolor)\n256)\n16)\nmono)\n${ARGS:-}\n";
        let errors = capture_script_contract_hits(missing_png);
        assert!(
            errors.iter().any(|error| error.contains("required BIN")),
            "{errors:?}"
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains("terminal dimensions")),
            "{errors:?}"
        );
        assert!(
            errors.iter().any(|error| error.contains("PNG conversion")),
            "{errors:?}"
        );

        let legacy_default = "BIN=${BIN:-target/debug/showcase}\n: \"${BIN:?x}\"";
        let errors = capture_script_contract_hits(legacy_default);
        assert!(
            errors
                .iter()
                .any(|error| error.contains("legacy showcase BIN default")),
            "the old implicit binary owner must fail: {errors:?}"
        );
    }

    #[test]
    fn capture_matrix_fails_closed_on_a_missing_cell() {
        let expected = capture_matrix_cases();
        let mut records: Vec<CaptureRecord> = expected
            .iter()
            .copied()
            .map(|case| CaptureRecord {
                case,
                path: PathBuf::from(format!("shots/{}.png", case.shot_name())),
                bytes: 1,
                sha256: "0".repeat(64),
            })
            .collect();
        records.pop();

        let error = validate_capture_records(&expected, &records)
            .expect_err("a missing matrix cell must fail closed");
        assert!(error.contains("missing capture cell(s)"), "{error}");
        assert!(error.contains("160x50"), "{error}");
    }

    #[test]
    fn capture_matrix_passes_theme_and_color_to_each_app() {
        let [_, _, app] = CAPTURE_APPS;
        let case = CaptureCase {
            app,
            width: 120,
            height: 40,
            color: "mono",
            theme: "paper",
        };
        assert_eq!(
            capture_arguments(case),
            "--motion paused --frame 0 --theme paper --color none"
        );
    }
}
