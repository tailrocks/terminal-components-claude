//! Read-only historical capture parity contract.
//!
//! The frozen archive is the oracle.  This module requires one explicit
//! mapping row and one provenance-bound replay record for every recipe before
//! it compares current output.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use serde_json::Value;
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

const HISTORICAL_MANIFEST: &str = "baseline/before/manifest.tsv";
const MAPPING_FILE: &str = "parity/recipes.tsv";
const EVIDENCE_FILE: &str = "parity/evidence.tsv";
const VISUAL_REVIEW_FILE: &str = "parity/visual_review.tsv";
const EXPECTED_RECIPE_COUNT: usize = 499;
const MAPPING_HEADER: &str = "recipe_id\tapp\tsurface\tviewport\tcolor\ttheme\tinitial_state\thistorical_command\tsteps\tcurrent_argv\texpected_txt\texpected_ansi\texpected_cursor\texpected_html\texpected_png\tcurrent_artifact_dir\tprovenance_path\ttrace_path\towner\treplay_policy";
const EVIDENCE_HEADER: &str = "recipe_id\treplay_status\tcurrent_revision\tsource_fingerprint\tdirty\tartifact_dir\tprovenance_path\ttrace_path\tvisual_review\treviewer";
const VISUAL_REVIEW_HEADER: &str = "recipe_id\tpng_path\tpng_sha256\thtml_path\thtml_sha256\treviewer\tdecision";
const ARTIFACTS: [(&str, &str); 5] = [
    ("txt", "txt"),
    ("ansi", "ansi"),
    ("cursor", "cursor"),
    ("html", "html"),
    ("png", "png"),
];
const COMPARED_ARTIFACTS: [&str; 3] = ["txt", "ansi", "cursor"];
const EMPTY_SHA256: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

#[derive(Clone, Debug, Eq, PartialEq)]
struct Viewport {
    width: u16,
    height: u16,
}

impl Viewport {
    fn parse(raw: &str) -> Result<Self, String> {
        let (width, height) = raw
            .split_once('x')
            .ok_or_else(|| format!("invalid viewport {}; expected WIDTHxHEIGHT", raw))?;
        let width = width
            .parse()
            .map_err(|_| format!("invalid viewport width {}", width))?;
        let height = height
            .parse()
            .map_err(|_| format!("invalid viewport height {}", height))?;
        if width == 0 || height == 0 {
            return Err(format!("viewport must be non-zero: {}", raw));
        }
        Ok(Self { width, height })
    }

    fn label(&self) -> String {
        format!("{}x{}", self.width, self.height)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ReplayStep {
    Keys(Vec<String>),
    Type(String),
    Mouse { kind: String, x: u16, y: u16 },
    Resize(Viewport),
    Wait(String),
    Anchor(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Recipe {
    id: String,
    app: String,
    viewport: Viewport,
    command: String,
    steps: String,
    stderr: String,
    parsed_steps: Vec<ReplayStep>,
}

impl Recipe {
    fn input_count(&self) -> usize {
        self.parsed_steps
            .iter()
            .map(|step| match step {
                ReplayStep::Keys(keys) => keys.len(),
                ReplayStep::Type(text) => text.chars().count(),
                ReplayStep::Mouse { .. } | ReplayStep::Resize(_) => 1,
                ReplayStep::Wait(_) | ReplayStep::Anchor(_) => 0,
            })
            .sum()
    }

    fn binary(&self) -> &'static str {
        match self.app.as_str() {
            "showcase" => "showcase",
            "tablepro" => "tablepro",
            "jackin" => "jackin-preview",
            _ => "",
        }
    }

    fn color(&self) -> &'static str {
        let args = self.command.split_whitespace().collect::<Vec<_>>();
        if args.windows(2).any(|pair| pair == ["--color", "none"]) {
            "mono"
        } else if args.windows(2).any(|pair| pair == ["--color", "256"]) {
            "256"
        } else if args.windows(2).any(|pair| pair == ["--color", "16"]) {
            "16"
        } else {
            "truecolor"
        }
    }

}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Mapping {
    recipe: Recipe,
    surface: String,
    initial_state: String,
    current_argv: Vec<String>,
    historical_paths: BTreeMap<String, String>,
    current_artifact_dir: String,
    provenance_path: String,
    trace_path: String,
    owner: String,
    replay_policy: String,
}

impl Mapping {
    fn current_path(&self, extension: &str) -> String {
        format!("{}/{}", self.current_artifact_dir, extension)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Evidence {
    recipe_id: String,
    replay_status: String,
    current_revision: String,
    source_fingerprint: String,
    dirty: bool,
    artifact_dir: String,
    provenance_path: String,
    trace_path: String,
    visual_review: String,
    reviewer: String,
}

struct LoadedContract {
    mappings: Vec<Mapping>,
    evidence: Vec<Evidence>,
}

/// Require and compare all current replay evidence.
pub(crate) fn contract(root: &Path) -> Result<(), String> {
    let mappings = load_mappings(root)?;
    let evidence = load_evidence(root, &mappings, false)?;
    let revision = head_revision(root)?;
    let source_fingerprint = source_fingerprint(root)?;
    let source_dirty = source_dirty(root)?;
    let loaded = LoadedContract { mappings, evidence };
    validate_evidence(root, &loaded, &source_fingerprint, source_dirty, true)?;
    validate_visual_review(root, &loaded, None)?;
    compare_all(root, &loaded)?;
    println!(
        "parity_contract: {} recipes compared against {}",
        loaded.mappings.len(),
        revision
    );
    Ok(())
}

/// Validate the frozen inventory, manifest parser, and explicit mapping only.
/// This never approves current output and is safe before replay capture.
pub(crate) fn dry_run(root: &Path) -> Result<(), String> {
    let mappings = load_mappings(root)?;
    let mut counts = BTreeMap::<&str, usize>::new();
    for mapping in &mappings {
        *counts.entry(mapping.recipe.app.as_str()).or_default() += 1;
    }
    let input_count = mappings
        .iter()
        .map(|mapping| mapping.recipe.input_count())
        .sum::<usize>();
    println!(
        "parity dry-run: {} recipes mapped; showcase={} tablepro={} jackin={}",
        mappings.len(),
        counts.get("showcase").copied().unwrap_or_default(),
        counts.get("tablepro").copied().unwrap_or_default(),
        counts.get("jackin").copied().unwrap_or_default()
    );
    println!("parity dry-run: {} replay input events parsed", input_count);
    println!(
        "parity dry-run: current replay evidence is required at {}; no output approved",
        EVIDENCE_FILE
    );
    Ok(())
}

/// Replay every mapped recipe through the real tmux capture harness.
pub(crate) fn replay(root: &Path) -> Result<(), String> {
    let script = root.join("tools/parity_replay.py");
    if !script.is_file() {
        return Err(format!(
            "parity replay script is missing: {}",
            script.display()
        ));
    }
    let status = Command::new("python3")
        .arg(&script)
        .current_dir(root)
        .status()
        .map_err(|error| format!("cannot start parity replay: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("parity replay failed with {status}"))
    }
}

/// Promote pending replay evidence only after exact artifact equality.
pub(crate) fn approve(root: &Path, reviewer: &str) -> Result<(), String> {
    if !safe_reviewer(reviewer) {
        return Err("parity approval reviewer is missing or unsafe".to_owned());
    }
    let mappings = load_mappings(root)?;
    let evidence = load_evidence(root, &mappings, true)?;
    let source_fingerprint = source_fingerprint(root)?;
    let source_dirty = source_dirty(root)?;
    let loaded = LoadedContract { mappings, evidence };
    validate_evidence(root, &loaded, &source_fingerprint, source_dirty, false)?;
    validate_visual_review(root, &loaded, Some(reviewer))?;
    let script = root.join("tools/parity_approve.py");
    if !script.is_file() {
        return Err(format!(
            "parity approval script is missing: {}",
            script.display()
        ));
    }
    let status = Command::new("python3")
        .arg(&script)
        .arg(reviewer)
        .current_dir(root)
        .status()
        .map_err(|error| format!("cannot start parity approval: {error}"))?;
    if status.success() {
        contract(root)
    } else {
        Err(format!("parity approval failed with {status}"))
    }
}

fn load_mappings(root: &Path) -> Result<Vec<Mapping>, String> {
    let manifest = read_text(root, HISTORICAL_MANIFEST, "historical manifest")?;
    let recipes = parse_manifest(&manifest)?;
    validate_frozen_inventory(root, &recipes)?;
    let mapping = read_text(root, MAPPING_FILE, "parity mapping")?;
    parse_mapping(&mapping, recipes)
}

fn parse_manifest(text: &str) -> Result<Vec<Recipe>, String> {
    let mut recipes = Vec::with_capacity(EXPECTED_RECIPE_COUNT);
    let mut seen = BTreeSet::new();
    let mut errors = Vec::new();
    for (line_index, line) in text.lines().enumerate() {
        let row = line_index + 1;
        if line.is_empty() {
            errors.push(format!("{}:{}: blank row", HISTORICAL_MANIFEST, row));
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() != 5 {
            errors.push(format!(
                "{}:{}: expected 5 fields, got {}",
                HISTORICAL_MANIFEST,
                row,
                fields.len()
            ));
            continue;
        }
        let [id, viewport, command, steps, stderr] = fields.as_slice() else {
            unreachable!("field count checked above");
        };
        if !safe_name(id) {
            errors.push(format!(
                "{}:{}: unsafe recipe id {}",
                HISTORICAL_MANIFEST, row, id
            ));
            continue;
        }
        if !seen.insert((*id).to_owned()) {
            errors.push(format!(
                "{}:{}: duplicate recipe {}",
                HISTORICAL_MANIFEST, row, id
            ));
        }
        let viewport = match Viewport::parse(viewport) {
            Ok(viewport) => viewport,
            Err(error) => {
                errors.push(format!("{}:{}: {}", HISTORICAL_MANIFEST, row, error));
                continue;
            }
        };
        let app = match command.split_whitespace().next() {
            Some("showcase") => "showcase",
            Some("tablepro") => "tablepro",
            Some("jackin-preview") => "jackin",
            Some(other) => {
                errors.push(format!(
                    "{}:{}: unknown application {}",
                    HISTORICAL_MANIFEST, row, other
                ));
                continue;
            }
            None => {
                errors.push(format!("{}:{}: empty command", HISTORICAL_MANIFEST, row));
                continue;
            }
        };
        if !stderr.ends_with(".log")
            || stderr.contains('/')
            || !safe_name(stderr.trim_end_matches(".log"))
        {
            errors.push(format!(
                "{}:{}: unsafe stderr label {}",
                HISTORICAL_MANIFEST, row, stderr
            ));
        }
        let parsed_steps = match parse_steps(steps) {
            Ok(steps) => steps,
            Err(error) => {
                errors.push(format!("{}:{}: {}", HISTORICAL_MANIFEST, row, error));
                Vec::new()
            }
        };
        recipes.push(Recipe {
            id: (*id).to_owned(),
            app: app.to_owned(),
            viewport,
            command: (*command).to_owned(),
            steps: (*steps).to_owned(),
            stderr: (*stderr).to_owned(),
            parsed_steps,
        });
    }
    if recipes.len() != EXPECTED_RECIPE_COUNT {
        errors.push(format!(
            "{}: {} rows found; expected {}",
            HISTORICAL_MANIFEST,
            recipes.len(),
            EXPECTED_RECIPE_COUNT
        ));
    }
    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }
    Ok(recipes)
}

fn parse_mapping(text: &str, recipes: Vec<Recipe>) -> Result<Vec<Mapping>, String> {
    let mut lines = text.lines();
    if lines.next() != Some(MAPPING_HEADER) {
        return Err(format!(
            "{} has an invalid header; expected {}",
            MAPPING_FILE, MAPPING_HEADER
        ));
    }
    let mut by_id = recipes
        .into_iter()
        .map(|recipe| (recipe.id.clone(), recipe))
        .collect::<BTreeMap<_, _>>();
    let mut mappings = Vec::with_capacity(EXPECTED_RECIPE_COUNT);
    let mut errors = Vec::new();
    for (line_index, line) in lines.enumerate() {
        let row = line_index + 2;
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() != 20 {
            errors.push(format!(
                "{}:{}: expected 20 fields, got {}",
                MAPPING_FILE,
                row,
                fields.len()
            ));
            continue;
        }
        let [
            id,
            app,
            surface,
            viewport,
            color,
            theme,
            initial_state,
            command,
            steps,
            current_argv,
            expected_txt,
            expected_ansi,
            expected_cursor,
            expected_html,
            expected_png,
            current_artifact_dir,
            provenance_path,
            trace_path,
            owner,
            replay_policy,
        ] = fields.as_slice()
        else {
            unreachable!("field count checked above");
        };
        let Some(recipe) = by_id.remove(*id) else {
            errors.push(format!(
                "{}:{}: unknown or duplicate recipe {}",
                MAPPING_FILE, row, id
            ));
            continue;
        };
        if *app != recipe.app {
            errors.push(format!("{}: mapping app differs from manifest", id));
        }
        if *viewport != recipe.viewport.label() {
            errors.push(format!("{}: mapping viewport differs from manifest", id));
        }
        if *command != recipe.command {
            errors.push(format!("{}: mapping command differs from manifest", id));
        }
        if *steps != recipe.steps {
            errors.push(format!("{}: mapping steps differ from manifest", id));
        }
        if *color != recipe.color() {
            errors.push(format!("{}: mapping color differs from manifest", id));
        }
        if *theme != "junie" {
            errors.push(format!("{}: historical theme must be junie", id));
        }
        if surface.is_empty() || surface.chars().any(char::is_whitespace) {
            errors.push(format!("{}: surface is empty or contains whitespace", id));
        }
        if initial_state.is_empty() || initial_state.chars().any(char::is_whitespace) {
            errors.push(format!(
                "{}: initial state is empty or contains whitespace",
                id
            ));
        }
        let argv = match parse_argv(current_argv) {
            Ok(argv) => argv,
            Err(error) => {
                errors.push(format!("{}: invalid current argv: {}", id, error));
                Vec::new()
            }
        };
        let expected_binary = format!("target/debug/{}", recipe.binary());
        if argv.first().map(String::as_str) != Some(expected_binary.as_str()) {
            errors.push(format!(
                "{}: current argv does not start with {}",
                id, expected_binary
            ));
        }
        let mut historical_paths = BTreeMap::new();
        for (extension, path) in [
            ("txt", *expected_txt),
            ("ansi", *expected_ansi),
            ("cursor", *expected_cursor),
            ("html", *expected_html),
            ("png", *expected_png),
        ] {
            let expected = format!("baseline/before/{}.{}", id, extension);
            if path != expected {
                errors.push(format!("{}: {} path must be {}", id, extension, expected));
            }
            historical_paths.insert(extension.to_owned(), path.to_owned());
        }
        let expected_dir = format!("parity/replays/{}", id);
        if *current_artifact_dir != expected_dir {
            errors.push(format!(
                "{}: current artifact directory must be {}",
                id, expected_dir
            ));
        }
        let expected_provenance = format!("{}/provenance.json", expected_dir);
        if *provenance_path != expected_provenance {
            errors.push(format!(
                "{}: provenance path must be {}",
                id, expected_provenance
            ));
        }
        let expected_trace = format!("{}/trace.json", expected_dir);
        if *trace_path != expected_trace {
            errors.push(format!("{}: trace path must be {}", id, expected_trace));
        }
        let expected_owner = format!("apps/{}", recipe.binary());
        if *owner != expected_owner {
            errors.push(format!("{}: owner must be {}", id, expected_owner));
        }
        if *replay_policy != "required" && !replay_policy.starts_with("reviewed-exception:") {
            errors.push(format!(
                "{}: replay policy is not required or reviewed-exception",
                id
            ));
        }
        mappings.push(Mapping {
            recipe,
            surface: (*surface).to_owned(),
            initial_state: (*initial_state).to_owned(),
            current_argv: argv,
            historical_paths,
            current_artifact_dir: (*current_artifact_dir).to_owned(),
            provenance_path: (*provenance_path).to_owned(),
            trace_path: (*trace_path).to_owned(),
            owner: (*owner).to_owned(),
            replay_policy: (*replay_policy).to_owned(),
        });
    }
    if !by_id.is_empty() {
        errors.push(format!(
            "{}: missing {} recipe row(s); first missing: {}",
            MAPPING_FILE,
            by_id.len(),
            by_id.keys().take(5).cloned().collect::<Vec<_>>().join(", ")
        ));
    }
    if mappings.len() != EXPECTED_RECIPE_COUNT {
        errors.push(format!(
            "{}: {} rows found; expected {}",
            MAPPING_FILE,
            mappings.len(),
            EXPECTED_RECIPE_COUNT
        ));
    }
    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }
    mappings.sort_by(|left, right| left.recipe.id.cmp(&right.recipe.id));
    Ok(mappings)
}

fn validate_frozen_inventory(root: &Path, recipes: &[Recipe]) -> Result<(), String> {
    let archive = root.join("baseline/before");
    validate_directory(&archive, HISTORICAL_MANIFEST)?;
    let expected = recipes
        .iter()
        .flat_map(|recipe| {
            ARTIFACTS
                .iter()
                .map(move |(_, extension)| format!("{}.{}", recipe.id, extension))
        })
        .collect::<BTreeSet<_>>();
    let allowed = BTreeSet::from([
        "MANIFEST.md".to_owned(),
        "NOTES.md".to_owned(),
        "manifest.tsv".to_owned(),
    ]);
    let mut actual = BTreeSet::new();
    for entry in WalkDir::new(&archive).follow_links(false) {
        let entry = entry.map_err(|error| format!("cannot inspect frozen archive: {}", error))?;
        if entry.path() == archive {
            continue;
        }
        let relative = entry
            .path()
            .strip_prefix(&archive)
            .map_err(|error| format!("cannot relativize frozen archive: {}", error))?
            .to_string_lossy()
            .replace('\\', "/");
        if entry.file_type().is_symlink() {
            return Err(format!(
                "frozen archive contains unsafe non-file entry baseline/before/{}",
                relative
            ));
        }
        if entry.file_type().is_dir() {
            if relative == "stderr" {
                continue;
            }
            return Err(format!(
                "frozen archive contains unexpected directory baseline/before/{}",
                relative
            ));
        }
        let is_stderr_log = relative
            .strip_prefix("stderr/")
            .and_then(|name| name.strip_suffix(".log"))
            .is_some_and(safe_name);
        if !entry.file_type().is_file()
            || (!allowed.contains(&relative) && !expected.contains(&relative) && !is_stderr_log)
        {
            return Err(format!(
                "frozen archive contains unexpected file baseline/before/{}",
                relative
            ));
        }
        if expected.contains(&relative) {
            actual.insert(relative);
        }
    }
    if actual != expected {
        let missing = expected
            .difference(&actual)
            .take(5)
            .cloned()
            .collect::<Vec<_>>();
        let extra = actual
            .difference(&expected)
            .take(5)
            .cloned()
            .collect::<Vec<_>>();
        return Err(format!(
            "frozen artifact inventory differs; missing={:?} extra={:?}",
            missing, extra
        ));
    }
    for recipe in recipes {
        for (_, extension) in ARTIFACTS {
            let relative = format!("baseline/before/{}.{}", recipe.id, extension);
            if read_bytes(root, &relative, "frozen artifact")?.is_empty() {
                return Err(format!("frozen artifact is empty: {}", relative));
            }
        }
    }
    Ok(())
}

fn load_evidence(
    root: &Path,
    mappings: &[Mapping],
    allow_pending_review: bool,
) -> Result<Vec<Evidence>, String> {
    let text = read_text(root, EVIDENCE_FILE, "parity replay evidence")?;
    let mut lines = text.lines();
    if lines.next() != Some(EVIDENCE_HEADER) {
        return Err(format!(
            "{} has an invalid header; expected {}",
            EVIDENCE_FILE, EVIDENCE_HEADER
        ));
    }
    let expected = mappings
        .iter()
        .map(|mapping| mapping.recipe.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    let mut evidence = Vec::with_capacity(mappings.len());
    let mut errors = Vec::new();
    for (line_index, line) in lines.enumerate() {
        let row = line_index + 2;
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() != 10 {
            errors.push(format!(
                "{}:{}: expected 10 fields, got {}",
                EVIDENCE_FILE,
                row,
                fields.len()
            ));
            continue;
        }
        let [
            recipe_id,
            replay_status,
            current_revision,
            source_fingerprint,
            dirty,
            artifact_dir,
            provenance_path,
            trace_path,
            visual_review,
            reviewer,
        ] = fields.as_slice()
        else {
            unreachable!("field count checked above");
        };
        if !expected.contains(recipe_id) {
            errors.push(format!(
                "{}:{}: unknown recipe {}",
                EVIDENCE_FILE, row, recipe_id
            ));
            continue;
        }
        if !seen.insert((*recipe_id).to_owned()) {
            errors.push(format!(
                "{}:{}: duplicate recipe {}",
                EVIDENCE_FILE, row, recipe_id
            ));
        }
        if !is_revision(current_revision) {
            errors.push(format!("{}:{}: invalid revision", EVIDENCE_FILE, row));
        }
        if !is_sha256(source_fingerprint) {
            errors.push(format!("{}:{}: invalid source fingerprint", EVIDENCE_FILE, row));
        }
        let dirty = match *dirty {
            "true" => true,
            "false" => false,
            _ => {
                errors.push(format!(
                    "{}:{}: dirty must be true or false",
                    EVIDENCE_FILE, row
                ));
                false
            }
        };
        for (label, path, prefix) in [
            ("artifact_dir", *artifact_dir, "parity/replays/"),
            ("provenance_path", *provenance_path, "parity/"),
            ("trace_path", *trace_path, "parity/"),
        ] {
            if let Err(error) = validate_relative_path(path, prefix) {
                errors.push(format!("{}:{}: {}: {}", EVIDENCE_FILE, row, label, error));
            }
        }
        if *replay_status != "ok" {
            errors.push(format!(
                "{}:{}: replay status is not ok",
                EVIDENCE_FILE, row
            ));
        }
        if !matches!(*visual_review, "approved" | "pending")
            || (!allow_pending_review && *visual_review != "approved")
        {
            errors.push(format!(
                "{}:{}: visual review state is invalid",
                EVIDENCE_FILE, row
            ));
        }
        if (*visual_review == "pending" && *reviewer != "pending")
            || (*visual_review == "approved"
                && (reviewer.is_empty() || reviewer.chars().any(char::is_whitespace)))
        {
            errors.push(format!(
                "{}:{}: reviewer is missing or unsafe",
                EVIDENCE_FILE, row
            ));
        }
        evidence.push(Evidence {
            recipe_id: (*recipe_id).to_owned(),
            replay_status: (*replay_status).to_owned(),
            current_revision: (*current_revision).to_owned(),
            source_fingerprint: (*source_fingerprint).to_owned(),
            dirty,
            artifact_dir: (*artifact_dir).to_owned(),
            provenance_path: (*provenance_path).to_owned(),
            trace_path: (*trace_path).to_owned(),
            visual_review: (*visual_review).to_owned(),
            reviewer: (*reviewer).to_owned(),
        });
    }
    if evidence.len() != EXPECTED_RECIPE_COUNT || seen.len() != EXPECTED_RECIPE_COUNT {
        errors.push(format!(
            "{}: {} rows found; expected {}",
            EVIDENCE_FILE,
            evidence.len(),
            EXPECTED_RECIPE_COUNT
        ));
    }
    let missing = expected
        .into_iter()
        .filter(|id| !seen.contains(*id))
        .take(5)
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        errors.push(format!(
            "{}: missing recipe rows; first missing {}",
            EVIDENCE_FILE,
            missing.join(", ")
        ));
    }
    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }
    evidence.sort_by(|left, right| left.recipe_id.cmp(&right.recipe_id));
    Ok(evidence)
}

fn validate_source_binding(
    recipe_id: &str,
    captured_revision: &str,
    captured_source_fingerprint: &str,
    captured_dirty: bool,
    current_source_fingerprint: &str,
    current_source_dirty: bool,
) -> Result<(), String> {
    if !is_revision(captured_revision) {
        return Err(format!("{}: evidence revision is invalid", recipe_id));
    }
    if captured_source_fingerprint != current_source_fingerprint {
        return Err(format!(
            "{}: evidence source fingerprint is not current",
            recipe_id
        ));
    }
    if captured_dirty != current_source_dirty {
        return Err(format!("{}: evidence dirty binding is stale", recipe_id));
    }
    Ok(())
}

fn validate_evidence(
    root: &Path,
    loaded: &LoadedContract,
    source_fingerprint: &str,
    source_dirty: bool,
    require_approved_review: bool,
) -> Result<(), String> {
    let mappings = loaded
        .mappings
        .iter()
        .map(|mapping| (mapping.recipe.id.as_str(), mapping))
        .collect::<BTreeMap<_, _>>();
    let mut errors = Vec::new();
    let mut captured_revisions = BTreeSet::new();
    for evidence in &loaded.evidence {
        let Some(mapping) = mappings.get(evidence.recipe_id.as_str()).copied() else {
            errors.push(format!(
                "evidence references unknown recipe {}",
                evidence.recipe_id
            ));
            continue;
        };
        if let Err(error) = validate_source_binding(
            &evidence.recipe_id,
            &evidence.current_revision,
            &evidence.source_fingerprint,
            evidence.dirty,
            source_fingerprint,
            source_dirty,
        ) {
            errors.push(error);
        }
        if is_revision(&evidence.current_revision) {
            captured_revisions.insert(evidence.current_revision.as_str());
        }
        if require_approved_review && evidence.visual_review != "approved" {
            errors.push(format!(
                "{}: visual review is not approved",
                evidence.recipe_id
            ));
        }
        if evidence.artifact_dir != mapping.current_artifact_dir
            || evidence.provenance_path != mapping.provenance_path
            || evidence.trace_path != mapping.trace_path
        {
            errors.push(format!(
                "{}: evidence paths differ from mapping",
                evidence.recipe_id
            ));
        }
        if let Err(error) = validate_generation(root, mapping, evidence) {
            errors.push(error);
        }
        if let Err(error) = validate_provenance(
            root,
            mapping,
            evidence,
            &evidence.current_revision,
            source_fingerprint,
        ) {
            errors.push(error);
        }
        if let Err(error) = validate_trace(
            root,
            mapping,
            evidence,
            &evidence.current_revision,
            source_fingerprint,
        ) {
            errors.push(error);
        }
    }
    for captured_revision in captured_revisions {
        match revision_exists(root, captured_revision) {
            Ok(true) => {}
            Ok(false) => errors.push(format!(
                "evidence revision {} is not a commit in this repository",
                captured_revision
            )),
            Err(error) => errors.push(error),
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

fn validate_generation(root: &Path, mapping: &Mapping, evidence: &Evidence) -> Result<(), String> {
    let directory = safe_join(root, &evidence.artifact_dir)?;
    validate_directory(&directory, &evidence.artifact_dir)?;
    for (_, extension) in ARTIFACTS {
        let path = mapping.current_path(extension);
        if read_bytes(root, &path, "current replay artifact")?.is_empty() {
            return Err(format!(
                "{}: empty current artifact {}",
                mapping.recipe.id, path
            ));
        }
    }
    Ok(())
}

fn validate_provenance(
    root: &Path,
    mapping: &Mapping,
    evidence: &Evidence,
    revision: &str,
    source_fingerprint: &str,
) -> Result<(), String> {
    let value = read_json(root, &evidence.provenance_path, "replay provenance")?;
    let record = provenance_record(&value, &mapping.recipe.id)?;
    let object = record
        .as_object()
        .ok_or_else(|| format!("{}: provenance record is not an object", mapping.recipe.id))?;
    for (field, expected) in [
        ("name", mapping.recipe.id.as_str()),
        ("app", mapping.recipe.binary()),
        ("theme", "junie"),
        ("color", mapping.recipe.color()),
        ("status", "ok"),
    ] {
        if object.get(field).and_then(Value::as_str) != Some(expected) {
            return Err(format!(
                "{}: provenance {} does not match",
                mapping.recipe.id, field
            ));
        }
    }
    if object.get("schema_version").and_then(Value::as_u64) != Some(2)
        || object.get("revision").and_then(Value::as_str) != Some(revision)
        || object.get("dirty").and_then(Value::as_bool) != Some(evidence.dirty)
    {
        return Err(format!(
            "{}: provenance identity is stale",
            mapping.recipe.id
        ));
    }
    let git_revision = object
        .get("git")
        .and_then(|git| git.get("revision"))
        .and_then(Value::as_str);
    let git_dirty = object
        .get("git")
        .and_then(|git| git.get("dirty"))
        .and_then(Value::as_bool);
    if git_revision != Some(revision) || git_dirty != Some(evidence.dirty) {
        return Err(format!(
            "{}: provenance git binding is stale",
            mapping.recipe.id
        ));
    }
    let parity = object
        .get("parity")
        .and_then(Value::as_object)
        .ok_or_else(|| format!("{}: parity provenance binding is missing", mapping.recipe.id))?;
    if parity.get("recipe_id").and_then(Value::as_str) != Some(mapping.recipe.id.as_str())
        || parity.get("revision").and_then(Value::as_str) != Some(revision)
        || parity
            .get("source_fingerprint")
            .and_then(Value::as_str)
            != Some(source_fingerprint)
        || parity.get("trace_path").and_then(Value::as_str) != Some(evidence.trace_path.as_str())
    {
        return Err(format!(
            "{}: parity provenance binding is stale",
            mapping.recipe.id
        ));
    }
    let expected_dimensions = (
        mapping.recipe.viewport.width,
        mapping.recipe.viewport.height,
    );
    if dimensions(object, "requested_dimensions") != Some(expected_dimensions)
        || dimensions(object, "dimensions") != Some(expected_dimensions)
    {
        return Err(format!(
            "{}: provenance dimensions differ",
            mapping.recipe.id
        ));
    }
    if !argv_equals(object.get("argv"), &mapping.current_argv)
        || !object
            .get("executed_argv")
            .and_then(Value::as_array)
            .is_some_and(|argv| !argv.is_empty() && argv.iter().all(Value::is_string))
    {
        return Err(format!("{}: provenance argv is invalid", mapping.recipe.id));
    }
    let artifacts = object
        .get("artifacts")
        .and_then(Value::as_object)
        .ok_or_else(|| format!("{}: provenance artifacts are missing", mapping.recipe.id))?;
    for (_, extension) in ARTIFACTS {
        let info = artifacts
            .get(extension)
            .and_then(Value::as_object)
            .ok_or_else(|| format!("{}: provenance lacks {}", mapping.recipe.id, extension))?;
        let path = mapping.current_path(extension);
        if info.get("path").and_then(Value::as_str) != Some(path.as_str())
            || info.get("status").and_then(Value::as_str) != Some("ok")
        {
            return Err(format!(
                "{}: provenance {} path/status is stale",
                mapping.recipe.id, extension
            ));
        }
        let bytes = read_bytes(root, &path, "current replay artifact")?;
        let (length, hash) = digest(&bytes);
        if info.get("bytes").and_then(Value::as_u64) != Some(length)
            || info.get("sha256").and_then(Value::as_str) != Some(hash.as_str())
        {
            return Err(format!(
                "{}: provenance {} hash is stale",
                mapping.recipe.id, extension
            ));
        }
    }
    let trace_bytes = read_bytes(root, &evidence.trace_path, "replay trace")?;
    let (_, trace_hash) = digest(&trace_bytes);
    if parity.get("trace_sha256").and_then(Value::as_str) != Some(trace_hash.as_str()) {
        return Err(format!(
            "{}: parity trace hash is stale",
            mapping.recipe.id
        ));
    }
    let stderr = object
        .get("stderr")
        .and_then(Value::as_object)
        .ok_or_else(|| format!("{}: provenance stderr is missing", mapping.recipe.id))?;
    if stderr.get("status").and_then(Value::as_str) != Some("empty")
        || stderr.get("bytes").and_then(Value::as_u64) != Some(0)
        || stderr.get("sha256").and_then(Value::as_str) != Some(EMPTY_SHA256)
    {
        return Err(format!("{}: replay stderr is not empty", mapping.recipe.id));
    }
    Ok(())
}

fn validate_trace(
    root: &Path,
    mapping: &Mapping,
    evidence: &Evidence,
    revision: &str,
    source_fingerprint: &str,
) -> Result<(), String> {
    let value = read_json(root, &evidence.trace_path, "replay trace")?;
    let object = value
        .as_object()
        .ok_or_else(|| format!("{}: trace is not an object", mapping.recipe.id))?;
    if object.get("schema_version").and_then(Value::as_u64) != Some(2)
        || object.get("recipe_id").and_then(Value::as_str) != Some(mapping.recipe.id.as_str())
        || object.get("revision").and_then(Value::as_str) != Some(revision)
        || object.get("source_fingerprint").and_then(Value::as_str) != Some(source_fingerprint)
    {
        return Err(format!("{}: trace identity is stale", mapping.recipe.id));
    }
    let steps = object
        .get("steps")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{}: trace steps are missing", mapping.recipe.id))?;
    let expected = mapping.recipe.parsed_steps.len() + 1;
    if steps.len() != expected {
        return Err(format!(
            "{}: trace has {} states; expected {}",
            mapping.recipe.id,
            steps.len(),
            expected
        ));
    }
    let initial = steps.first().and_then(Value::as_object).ok_or_else(|| {
        format!("{}: trace initial state is missing", mapping.recipe.id)
    })?;
    if initial.get("index").and_then(Value::as_u64) != Some(0)
        || initial.get("event").and_then(Value::as_str) != Some("initial")
        || !initial
            .get("state_sha256")
            .and_then(Value::as_str)
            .is_some_and(is_sha256)
    {
        return Err(format!(
            "{}: trace initial state is invalid",
            mapping.recipe.id
        ));
    }
    for (index, (expected_step, step)) in mapping
        .recipe
        .parsed_steps
        .iter()
        .zip(steps.iter().skip(1))
        .enumerate()
    {
        let step = step.as_object().ok_or_else(|| {
            format!(
                "{}: trace step {} is not an object",
                mapping.recipe.id, index
            )
        })?;
        let expected_index = index + 1;
        let expected_kind = match expected_step {
            ReplayStep::Keys(_) => "keys",
            ReplayStep::Type(_) => "type",
            ReplayStep::Mouse { .. } => "mouse",
            ReplayStep::Resize(_) => "resize",
            ReplayStep::Wait(_) => "wait",
            ReplayStep::Anchor(_) => "anchor",
        };
        if step.get("index").and_then(Value::as_u64) != Some(expected_index as u64)
            || step.get("step_index").and_then(Value::as_u64) != Some(index as u64)
            || step.get("kind").and_then(Value::as_str) != Some(expected_kind)
            || step.get("observed").and_then(Value::as_bool) != Some(true)
            || !step
                .get("event")
                .and_then(Value::as_str)
                .is_some_and(|event| event.starts_with(&format!("{}:", expected_kind)))
            || !step
                .get("state_sha256")
                .and_then(Value::as_str)
                .is_some_and(is_sha256)
        {
            return Err(format!(
                "{}: trace step {} lacks a state fingerprint",
                mapping.recipe.id, index
            ));
        }
        if let ReplayStep::Anchor(expected_anchor) = expected_step
            && step.get("anchor").and_then(Value::as_str) != Some(expected_anchor.as_str())
        {
            return Err(format!(
                "{}: trace anchor {} was not preserved",
                mapping.recipe.id, index
            ));
        }
    }
    Ok(())
}

fn validate_visual_review(
    root: &Path,
    loaded: &LoadedContract,
    approval_reviewer: Option<&str>,
) -> Result<(), String> {
    let text = read_text(root, VISUAL_REVIEW_FILE, "independent visual review")?;
    let mut lines = text.lines();
    if lines.next() != Some(VISUAL_REVIEW_HEADER) {
        return Err(format!(
            "{} has an invalid header; expected {}",
            VISUAL_REVIEW_FILE, VISUAL_REVIEW_HEADER
        ));
    }
    let mappings = loaded
        .mappings
        .iter()
        .map(|mapping| (mapping.recipe.id.as_str(), mapping))
        .collect::<BTreeMap<_, _>>();
    let evidence = loaded
        .evidence
        .iter()
        .map(|evidence| (evidence.recipe_id.as_str(), evidence))
        .collect::<BTreeMap<_, _>>();
    let expected = loaded
        .mappings
        .iter()
        .map(|mapping| mapping.recipe.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    let mut errors = Vec::new();
    for (line_index, line) in lines.enumerate() {
        let row = line_index + 2;
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() != 7 {
            errors.push(format!(
                "{}:{}: expected 7 fields, got {}",
                VISUAL_REVIEW_FILE,
                row,
                fields.len()
            ));
            continue;
        }
        let [recipe_id, png_path, png_sha256, html_path, html_sha256, reviewer, decision] =
            fields.as_slice()
        else {
            unreachable!("field count checked above");
        };
        let Some(mapping) = mappings.get(recipe_id).copied() else {
            errors.push(format!(
                "{}:{}: unknown recipe {}",
                VISUAL_REVIEW_FILE, row, recipe_id
            ));
            continue;
        };
        if !seen.insert(*recipe_id) {
            errors.push(format!(
                "{}:{}: duplicate review record for {}",
                VISUAL_REVIEW_FILE, row, recipe_id
            ));
        }
        for (artifact, path, sha256) in [
            ("png", *png_path, *png_sha256),
            ("html", *html_path, *html_sha256),
        ] {
            let expected_path = mapping.current_path(artifact);
            if path != expected_path {
                errors.push(format!(
                    "{}:{}: {} review path differs from mapping",
                    VISUAL_REVIEW_FILE, row, artifact
                ));
            }
            if !is_sha256(sha256) {
                errors.push(format!(
                    "{}:{}: {} review hash is invalid",
                    VISUAL_REVIEW_FILE, row, artifact
                ));
            } else if let Ok(bytes) = read_bytes(root, path, "visual review artifact") {
                let (_, actual) = digest(&bytes);
                if actual != sha256 {
                    errors.push(format!(
                        "{}:{}: {} visual review hash is stale",
                        VISUAL_REVIEW_FILE, row, artifact
                    ));
                }
            } else {
                errors.push(format!(
                    "{}:{}: {} visual review artifact is missing",
                    VISUAL_REVIEW_FILE, row, artifact
                ));
            }
        }
        if *decision != "approved" || !safe_reviewer(reviewer) {
            errors.push(format!(
                "{}:{}: visual review decision or reviewer is invalid",
                VISUAL_REVIEW_FILE, row
            ));
        }
        if let Some(approval_reviewer) = approval_reviewer
            && *reviewer == approval_reviewer
        {
            errors.push(format!(
                "{}:{}: visual review must be independent of approval",
                VISUAL_REVIEW_FILE, row
            ));
        }
        if approval_reviewer.is_none()
            && evidence
                .get(recipe_id)
                .is_some_and(|evidence| evidence.reviewer == *reviewer)
        {
            errors.push(format!(
                "{}:{}: visual review must be independent of evidence approval",
                VISUAL_REVIEW_FILE, row
            ));
        }
    }
    if seen.len() != expected.len() || seen != expected {
        errors.push(format!(
            "{}: expected {} unique recipe review records covering PNG and HTML, found {}",
            VISUAL_REVIEW_FILE,
            expected.len(),
            seen.len()
        ));
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

fn compare_all(root: &Path, loaded: &LoadedContract) -> Result<(), String> {
    let evidence = loaded
        .evidence
        .iter()
        .map(|evidence| (evidence.recipe_id.as_str(), evidence))
        .collect::<BTreeMap<_, _>>();
    for mapping in &loaded.mappings {
        let evidence = evidence
            .get(mapping.recipe.id.as_str())
            .copied()
            .ok_or_else(|| format!("{}: evidence disappeared", mapping.recipe.id))?;
        for extension in COMPARED_ARTIFACTS {
            let expected = mapping.historical_paths.get(extension).ok_or_else(|| {
                format!("{}: missing historical {}", mapping.recipe.id, extension)
            })?;
            let current = format!("{}/{}", evidence.artifact_dir, extension);
            let expected_bytes = read_bytes(root, expected, "historical artifact")?;
            let current_bytes = read_bytes(root, &current, "current replay artifact")?;
            validate_dimensions(&mapping.recipe, extension, &expected_bytes, "historical")?;
            validate_dimensions(&mapping.recipe, extension, &current_bytes, "current")?;
            if expected_bytes != current_bytes {
                return Err(first_difference(
                    &mapping.recipe,
                    mapping,
                    extension,
                    &expected_bytes,
                    &current_bytes,
                ));
            }
        }
    }
    Ok(())
}

fn first_difference(
    recipe: &Recipe,
    mapping: &Mapping,
    extension: &str,
    expected: &[u8],
    actual: &[u8],
) -> String {
    let offset = expected
        .iter()
        .zip(actual.iter())
        .position(|(left, right)| left != right)
        .unwrap_or(expected.len().min(actual.len()));
    let (x, y) = match extension {
        "txt" => text_position(actual, offset),
        "ansi" => ansi_position(actual, offset),
        "cursor" => cursor_position(actual),
        _ => (0, 0),
    };
    format!(
        "parity mismatch: recipe={} owner={} artifact={} coordinates=({}, {}) byte_offset={} expected={} actual={} historical={} current={}",
        recipe.id,
        mapping.owner,
        extension,
        x,
        y,
        offset,
        byte_value(expected.get(offset).copied()),
        byte_value(actual.get(offset).copied()),
        mapping
            .historical_paths
            .get(extension)
            .map(String::as_str)
            .unwrap_or("missing"),
        mapping.current_path(extension),
    )
}

fn validate_dimensions(
    recipe: &Recipe,
    extension: &str,
    bytes: &[u8],
    side: &str,
) -> Result<(), String> {
    if extension == "cursor" {
        let fields = String::from_utf8_lossy(bytes)
            .split_whitespace()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        if fields.len() != 3 {
            return Err(format!(
                "{}: {} cursor must be x y visible",
                recipe.id, side
            ));
        }
        let x = fields[0]
            .parse::<u16>()
            .map_err(|_| format!("{}: {} cursor x invalid", recipe.id, side))?;
        let y = fields[1]
            .parse::<u16>()
            .map_err(|_| format!("{}: {} cursor y invalid", recipe.id, side))?;
        if x > recipe.viewport.width
            || y >= recipe.viewport.height
            || !matches!(fields[2].as_str(), "0" | "1")
        {
            return Err(format!(
                "{}: {} cursor is outside {}",
                recipe.id,
                side,
                recipe.viewport.label()
            ));
        }
        return Ok(());
    }
    if matches!(extension, "txt" | "ansi") {
        let rows = bytes.iter().filter(|byte| **byte == b'\n').count();
        if rows != usize::from(recipe.viewport.height) {
            return Err(format!(
                "{}: {} {} has {} rows; expected {}",
                recipe.id, side, extension, rows, recipe.viewport.height
            ));
        }
    }
    Ok(())
}

fn text_position(bytes: &[u8], offset: usize) -> (usize, usize) {
    let prefix = &bytes[..offset.min(bytes.len())];
    let y = prefix.iter().filter(|byte| **byte == b'\n').count();
    let start = prefix
        .iter()
        .rposition(|byte| *byte == b'\n')
        .map_or(0, |position| position + 1);
    (String::from_utf8_lossy(&prefix[start..]).chars().count(), y)
}

fn ansi_position(bytes: &[u8], offset: usize) -> (usize, usize) {
    let limit = offset.min(bytes.len());
    let mut index = 0;
    let mut x = 0;
    let mut y = 0;
    while index < limit {
        match bytes[index] {
            b'\n' => {
                y += 1;
                x = 0;
                index += 1;
            }
            b'\r' => {
                x = 0;
                index += 1;
            }
            0x1b if index + 1 < limit && bytes[index + 1] == b'[' => {
                index += 2;
                while index < limit {
                    let byte = bytes[index];
                    index += 1;
                    if (0x40..=0x7e).contains(&byte) {
                        break;
                    }
                }
            }
            byte if byte.is_ascii_control() => index += 1,
            byte if byte & 0b1100_0000 == 0b1000_0000 => index += 1,
            byte if byte & 0b1000_0000 != 0 => {
                let width = if byte & 0b1110_0000 == 0b1100_0000 {
                    2
                } else if byte & 0b1111_0000 == 0b1110_0000 {
                    3
                } else {
                    4
                };
                index = (index + width).min(limit);
                x += 1;
            }
            _ => {
                x += 1;
                index += 1;
            }
        }
    }
    (x, y)
}

fn cursor_position(bytes: &[u8]) -> (usize, usize) {
    let fields = String::from_utf8_lossy(bytes)
        .split_whitespace()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if fields.len() >= 2
        && let (Ok(x), Ok(y)) = (fields[0].parse(), fields[1].parse())
    {
        return (x, y);
    }
    (0, 0)
}

fn byte_value(byte: Option<u8>) -> String {
    byte.map_or_else(|| "eof".to_owned(), |byte| format!("0x{:02x}", byte))
}

fn parse_steps(raw: &str) -> Result<Vec<ReplayStep>, String> {
    if raw == "(none)" {
        return Ok(Vec::new());
    }
    let mut steps = Vec::new();
    for segment in split_steps(raw)? {
        if let Some(inner) = segment
            .strip_prefix("keys(")
            .and_then(|value| value.strip_suffix(')'))
        {
            let keys = inner
                .split_whitespace()
                .map(str::to_owned)
                .collect::<Vec<_>>();
            if keys.is_empty() {
                return Err(format!("empty keys step {}", segment));
            }
            steps.push(ReplayStep::Keys(keys));
        } else if let Some(inner) = segment
            .strip_prefix("type(\"")
            .and_then(|value| value.strip_suffix("\")"))
        {
            steps.push(ReplayStep::Type(unescape(inner)?));
        } else if let Some(inner) = segment
            .strip_prefix("mouse(")
            .and_then(|value| value.strip_suffix(')'))
        {
            let mut fields = inner.split_whitespace();
            let kind = fields
                .next()
                .ok_or_else(|| format!("invalid mouse step {}", segment))?;
            let coordinate = fields
                .next()
                .ok_or_else(|| format!("invalid mouse step {}", segment))?;
            if fields.next().is_some()
                || !matches!(
                    kind,
                    "move" | "click" | "rclick" | "down" | "up" | "drag" | "wheelup" | "wheeldown"
                )
            {
                return Err(format!("invalid mouse step {}", segment));
            }
            let (x, y) = coordinate
                .split_once(',')
                .ok_or_else(|| format!("invalid mouse coordinate {}", coordinate))?;
            let x = x.parse().map_err(|_| format!("invalid mouse x {}", x))?;
            let y = y.parse().map_err(|_| format!("invalid mouse y {}", y))?;
            if x == 0 || y == 0 {
                return Err(format!("mouse coordinates are 1-based {}", coordinate));
            }
            steps.push(ReplayStep::Mouse {
                kind: kind.to_owned(),
                x,
                y,
            });
        } else if let Some(inner) = segment
            .strip_prefix("resize(")
            .and_then(|value| value.strip_suffix(')'))
        {
            steps.push(ReplayStep::Resize(Viewport::parse(inner)?));
        } else if let Some(inner) = segment
            .strip_prefix("wait(")
            .and_then(|value| value.strip_suffix(')'))
        {
            if !inner.ends_with('s')
                || inner.len() == 1
                || inner[..inner.len() - 1].parse::<f64>().is_err()
            {
                return Err(format!("invalid wait step {}", segment));
            }
            steps.push(ReplayStep::Wait(inner.to_owned()));
        } else if let Some(inner) = segment
            .strip_prefix("(on \"")
            .and_then(|value| value.strip_suffix("\")"))
        {
            steps.push(ReplayStep::Anchor(unescape(inner)?));
        } else {
            return Err(format!("unrecognized replay step {}", segment));
        }
    }
    Ok(steps)
}

fn split_steps(raw: &str) -> Result<Vec<String>, String> {
    let mut result = Vec::new();
    let mut start = 0;
    let mut depth = 0usize;
    let mut quoted = false;
    let mut escaped = false;
    let bytes = raw.as_bytes();
    let separator = " · ".as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        if quoted {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                quoted = false;
            }
            index += 1;
            continue;
        }
        match byte {
            b'"' => quoted = true,
            b'(' => depth += 1,
            b')' if depth == 0 => return Err(format!("unbalanced steps {}", raw)),
            b')' => depth -= 1,
            _ => {}
        }
        if depth == 0 && bytes.get(index..index + separator.len()) == Some(separator) {
            let segment = raw[start..index].trim();
            if segment.is_empty() {
                return Err(format!("empty step {}", raw));
            }
            result.push(segment.to_owned());
            index += separator.len();
            start = index;
            continue;
        }
        index += 1;
    }
    if quoted || depth != 0 {
        return Err(format!("unbalanced steps {}", raw));
    }
    let segment = raw[start..].trim();
    if segment.is_empty() {
        return Err(format!("empty step {}", raw));
    }
    result.push(segment.to_owned());
    Ok(result)
}

fn unescape(raw: &str) -> Result<String, String> {
    let mut output = String::new();
    let mut escaped = false;
    for character in raw.chars() {
        if escaped {
            match character {
                '"' | '\\' => output.push(character),
                'n' => output.push('\n'),
                'r' => output.push('\r'),
                't' => output.push('\t'),
                other => return Err(format!("unsupported escape {}", other)),
            }
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else {
            output.push(character);
        }
    }
    if escaped {
        return Err("unterminated escape".to_owned());
    }
    Ok(output)
}

fn parse_argv(raw: &str) -> Result<Vec<String>, String> {
    let value: Value = serde_json::from_str(raw).map_err(|error| error.to_string())?;
    let values = value
        .as_array()
        .ok_or_else(|| "current argv is not an array".to_owned())?;
    if values.is_empty() || values.iter().any(|value| !value.is_string()) {
        return Err("current argv must be a non-empty string array".to_owned());
    }
    Ok(values
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect())
}

fn read_text(root: &Path, relative: &str, label: &str) -> Result<String, String> {
    String::from_utf8(read_bytes(root, relative, label)?)
        .map_err(|error| format!("{} {} is not UTF-8: {}", label, relative, error))
}

fn read_json(root: &Path, relative: &str, label: &str) -> Result<Value, String> {
    let text = read_text(root, relative, label)?;
    serde_json::from_str(&text)
        .map_err(|error| format!("{} {} is not valid JSON: {}", label, relative, error))
}

fn read_bytes(root: &Path, relative: &str, label: &str) -> Result<Vec<u8>, String> {
    let path = safe_join(root, relative)?;
    let metadata = fs::symlink_metadata(&path)
        .map_err(|error| format!("cannot inspect {} {}: {}", label, relative, error))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!("{} is not a regular file: {}", label, relative));
    }
    let mut file = super::open_capture_artifact(&path)?;
    let opened = file
        .metadata()
        .map_err(|error| format!("cannot inspect opened {} {}: {}", label, relative, error))?;
    if !opened.is_file() {
        return Err(format!(
            "opened {} is not a regular file: {}",
            label, relative
        ));
    }
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|error| format!("cannot read {} {}: {}", label, relative, error))?;
    Ok(bytes)
}

fn safe_join(root: &Path, relative: &str) -> Result<PathBuf, String> {
    validate_relative_path(relative, "")?;
    let mut path = root.to_owned();
    for component in Path::new(relative).components() {
        let Component::Normal(component) = component else {
            return Err(format!("unsafe relative path {}", relative));
        };
        path.push(component);
    }
    Ok(path)
}

fn validate_relative_path(path: &str, prefix: &str) -> Result<(), String> {
    if path.is_empty()
        || Path::new(path).is_absolute()
        || (!prefix.is_empty() && !path.starts_with(prefix))
        || Path::new(path)
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!("unsafe relative path {}", path));
    }
    Ok(())
}

fn validate_directory(path: &Path, label: &str) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("cannot inspect {}: {}", label, error))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!(
            "{} is not a real directory {}",
            label,
            path.display()
        ));
    }
    Ok(())
}

fn provenance_record(value: &Value, recipe_id: &str) -> Result<Value, String> {
    match value {
        Value::Object(object) => Ok(Value::Object(object.clone())),
        Value::Array(records) => {
            let matches = records
                .iter()
                .filter(|record| record.get("name").and_then(Value::as_str) == Some(recipe_id))
                .collect::<Vec<_>>();
            if matches.len() != 1 {
                return Err(format!(
                    "{}: provenance has {} matching records",
                    recipe_id,
                    matches.len()
                ));
            }
            Ok(matches[0].clone())
        }
        _ => Err(format!(
            "{}: provenance is not an object or array",
            recipe_id
        )),
    }
}

fn dimensions(object: &serde_json::Map<String, Value>, key: &str) -> Option<(u16, u16)> {
    let value = object.get(key)?.as_object()?;
    Some((
        value.get("columns")?.as_u64()?.try_into().ok()?,
        value.get("rows")?.as_u64()?.try_into().ok()?,
    ))
}

fn argv_equals(value: Option<&Value>, expected: &[String]) -> bool {
    value.and_then(Value::as_array).is_some_and(|actual| {
        actual.len() == expected.len()
            && actual
                .iter()
                .zip(expected)
                .all(|(actual, expected)| actual.as_str() == Some(expected.as_str()))
    })
}

fn digest(bytes: &[u8]) -> (u64, String) {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    (bytes.len() as u64, format!("{:x}", hasher.finalize()))
}

fn source_fingerprint(root: &Path) -> Result<String, String> {
    let output = Command::new("git")
        .current_dir(root)
        .args(["ls-files", "-co", "--exclude-standard", "-z"])
        .output()
        .map_err(|error| format!("cannot list source files: {}", error))?;
    if !output.status.success() {
        return Err(format!(
            "cannot list source files: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let mut paths = output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
        .map(|path| {
            String::from_utf8(path.to_vec())
                .map_err(|error| format!("source path is not UTF-8: {}", error))
        })
        .collect::<Result<Vec<_>, _>>()?;
    paths.retain(|path| source_path_allowed(path));
    paths.sort();
    paths.dedup();

    let mut hasher = Sha256::new();
    for relative in paths {
        let bytes = read_bytes(root, &relative, "source file")?;
        hasher.update(relative.as_bytes());
        hasher.update([0]);
        hasher.update(bytes.len().to_string().as_bytes());
        hasher.update([0]);
        hasher.update(bytes);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn source_path_allowed(path: &str) -> bool {
    let normalized = path.replace('\\', "/");
    if normalized.is_empty()
        || normalized == ".DS_Store"
        || normalized.split('/').any(|part| part == ".DS_Store")
    {
        return false;
    }
    if normalized
        .split('/')
        .any(|part| part == "target" || part.starts_with(".codex-target-"))
    {
        return false;
    }
    normalized == "parity/recipes.tsv" || !normalized.starts_with("parity/")
}

fn source_dirty(root: &Path) -> Result<bool, String> {
    let output = Command::new("git")
        .current_dir(root)
        .args(["status", "--porcelain=v1", "--untracked-files=all", "-z"])
        .output()
        .map_err(|error| format!("cannot inspect Git source dirtiness: {}", error))?;
    if !output.status.success() {
        return Err(format!(
            "cannot inspect Git source dirtiness: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    source_dirty_from_status(&output.stdout)
}

fn source_dirty_from_status(status: &[u8]) -> Result<bool, String> {
    let mut fields = status
        .split(|byte| *byte == 0)
        .filter(|field| !field.is_empty());
    while let Some(record) = fields.next() {
        if record.len() < 4 || record[2] != b' ' {
            return Err("Git status output is malformed".to_owned());
        }
        let status_code = [record[0], record[1]];
        if status_code.iter().any(|byte| {
            !matches!(
                byte,
                b' ' | b'M' | b'A' | b'D' | b'R' | b'C' | b'T' | b'U' | b'?' | b'!'
            )
        }) {
            return Err("Git status output contains an unknown status".to_owned());
        }
        let path = std::str::from_utf8(&record[3..])
            .map_err(|error| format!("Git status path is not UTF-8: {}", error))?;
        if source_path_allowed(path) {
            return Ok(true);
        }
        if matches!(status_code[0], b'R' | b'C') {
            let destination = fields
                .next()
                .ok_or_else(|| "Git status rename record is incomplete".to_owned())?;
            let destination = std::str::from_utf8(destination)
                .map_err(|error| format!("Git status path is not UTF-8: {}", error))?;
            if source_path_allowed(destination) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn head_revision(root: &Path) -> Result<String, String> {
    let output = Command::new("git")
        .current_dir(root)
        .args(["rev-parse", "--verify", "HEAD"])
        .output()
        .map_err(|error| format!("cannot read current Git revision: {}", error))?;
    if !output.status.success() {
        return Err(format!(
            "cannot read current Git revision: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let revision = String::from_utf8(output.stdout)
        .map_err(|error| format!("Git revision is not UTF-8: {}", error))?
        .trim()
        .to_owned();
    if !is_revision(&revision) {
        return Err(format!("current Git revision is not full: {}", revision));
    }
    Ok(revision)
}

fn revision_exists(root: &Path, revision: &str) -> Result<bool, String> {
    if !is_revision(revision) {
        return Ok(false);
    }
    let object = format!("{}^{{commit}}", revision);
    let status = Command::new("git")
        .current_dir(root)
        .args(["rev-parse", "--verify", "--quiet", &object])
        .status()
        .map_err(|error| format!("cannot validate evidence revision: {}", error))?;
    Ok(status.success())
}

fn is_revision(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn safe_name(value: &str) -> bool {
    let mut characters = value.chars();
    matches!(characters.next(), Some(first) if first.is_ascii_alphanumeric())
        && characters.all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.')
        })
}

fn safe_reviewer(value: &str) -> bool {
    let mut characters = value.chars();
    matches!(characters.next(), Some(first) if first.is_ascii_alphanumeric())
        && characters.all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '-')
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace root")
            .to_owned()
    }

    #[test]
    fn frozen_manifest_and_mapping_cover_all_recipes() {
        let mappings = load_mappings(&workspace_root()).expect("mapping contract");
        assert_eq!(mappings.len(), EXPECTED_RECIPE_COUNT);
        assert_eq!(
            mappings
                .iter()
                .filter(|mapping| mapping.recipe.app == "showcase")
                .count(),
            208
        );
        assert_eq!(
            mappings
                .iter()
                .filter(|mapping| mapping.recipe.app == "tablepro")
                .count(),
            108
        );
        assert_eq!(
            mappings
                .iter()
                .filter(|mapping| mapping.recipe.app == "jackin")
                .count(),
            183
        );
    }

    #[test]
    fn all_historical_steps_parse() {
        let text = read_text(
            &workspace_root(),
            HISTORICAL_MANIFEST,
            "historical manifest",
        )
        .expect("manifest");
        let recipes = parse_manifest(&text).expect("steps");
        assert!(recipes.iter().any(|recipe| recipe.input_count() > 0));
    }

    #[test]
    fn input_count_ignores_observation_steps() {
        let parsed_steps = parse_steps(
            r#"keys(Tab Enter) · type("ab") · mouse(click 1,1) · resize(80x24) · wait(1s) · (on "ready")"#,
        )
        .expect("steps");
        let recipe = Recipe {
            id: "test".to_owned(),
            app: "showcase".to_owned(),
            viewport: Viewport {
                width: 80,
                height: 24,
            },
            command: "showcase --page test".to_owned(),
            steps: "test".to_owned(),
            stderr: "test.log".to_owned(),
            parsed_steps,
        };
        assert_eq!(recipe.input_count(), 6);
    }

    #[test]
    fn source_dirty_ignores_generated_parity_outputs() {
        assert_eq!(
            source_dirty_from_status(b"?? parity/replays/example/txt\0"),
            Ok(false)
        );
        assert_eq!(
            source_dirty_from_status(b"?? parity/visual_review.tsv\0"),
            Ok(false)
        );
        assert_eq!(
            source_dirty_from_status(b"?? parity/recipes.tsv\0"),
            Ok(true)
        );
        assert_eq!(
            source_dirty_from_status(b" M crates/tui/src/lib.rs\0"),
            Ok(true)
        );
    }

    #[test]
    fn source_dirty_fails_closed_on_malformed_status() {
        let error = source_dirty_from_status(b" M").expect_err("malformed status");
        assert!(error.contains("malformed"), "{}", error);
        let error = source_dirty_from_status(b"Z  parity/output\0").expect_err("unknown status");
        assert!(error.contains("unknown"), "{}", error);
    }

    #[test]
    fn source_binding_accepts_revision_changes_with_same_source_state() {
        let revision = "a".repeat(40);
        let fingerprint = "b".repeat(64);
        assert!(
            validate_source_binding(
                "recipe",
                &revision,
                &fingerprint,
                false,
                &fingerprint,
                false,
            )
            .is_ok()
        );
        assert!(
            validate_source_binding(
                "recipe",
                &revision,
                &"c".repeat(64),
                false,
                &fingerprint,
                false,
            )
            .is_err()
        );
        assert!(
            validate_source_binding("recipe", &revision, &fingerprint, true, &fingerprint, false,)
                .is_err()
        );
    }

    #[test]
    fn evidence_revision_is_a_real_commit() {
        let revision = head_revision(&workspace_root()).expect("HEAD");
        assert!(revision_exists(&workspace_root(), &revision).expect("revision lookup"));
        assert!(!revision_exists(&workspace_root(), &"0".repeat(40)).expect("revision lookup"));
    }

    #[test]
    fn unknown_replay_step_fails_closed() {
        let error = parse_steps("keys(Tab) · explode(now)").expect_err("unknown step");
        assert!(error.contains("unrecognized replay step"), "{}", error);
    }

    #[test]
    fn first_difference_reports_owner_artifact_and_coordinates() {
        let recipe = Recipe {
            id: "showcase_test_4x2".to_owned(),
            app: "showcase".to_owned(),
            viewport: Viewport {
                width: 4,
                height: 2,
            },
            command: "showcase --page test".to_owned(),
            steps: "(none)".to_owned(),
            stderr: "001_showcase.log".to_owned(),
            parsed_steps: Vec::new(),
        };
        let mapping = Mapping {
            recipe: recipe.clone(),
            surface: "test".to_owned(),
            initial_state: "default".to_owned(),
            current_argv: vec![
                "target/debug/showcase".to_owned(),
                "--page".to_owned(),
                "test".to_owned(),
            ],
            historical_paths: BTreeMap::from([(
                "txt".to_owned(),
                "baseline/before/showcase_test_4x2.txt".to_owned(),
            )]),
            current_artifact_dir: "parity/replays/showcase_test_4x2".to_owned(),
            provenance_path: "parity/replays/showcase_test_4x2/provenance.json".to_owned(),
            trace_path: "parity/replays/showcase_test_4x2/trace.json".to_owned(),
            owner: "apps/showcase".to_owned(),
            replay_policy: "required".to_owned(),
        };
        let error = first_difference(&recipe, &mapping, "txt", b"abcd\nefgh\n", b"abXd\nefgh\n");
        assert!(error.contains("recipe=showcase_test_4x2"), "{}", error);
        assert!(error.contains("owner=apps/showcase"), "{}", error);
        assert!(error.contains("artifact=txt"), "{}", error);
        assert!(error.contains("coordinates=(2, 0)"), "{}", error);
    }

    #[test]
    fn ansi_coordinates_ignore_sgr() {
        assert_eq!(ansi_position(b"\x1b[31mab\ncd\n", 7), (2, 0));
    }
}
