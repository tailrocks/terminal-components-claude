use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const SCHEMA: &str = "termrock-live-binary-source.v1";
const MANIFEST_FILE: &str = "source-manifest.json";
const REQUIRED_BINARIES: [(&str, &str); 4] = [
    ("showcase", "showcase"),
    ("holla", "holla"),
    ("jackin", "jackin-preview"),
    ("tablepro", "tablepro"),
];
const HASH_EVIDENCE_NAMES: [&str; 5] =
    ["showcase", "holla", "jackin-preview", "tablepro", "jackin"];

pub fn load_verified(bin_dir: &Path) -> Result<Value, Box<dyn Error>> {
    let bin_dir = canonical_directory(bin_dir)?;
    let path = bin_dir.join(MANIFEST_FILE);
    let bytes = fs::read(&path).map_err(|error| {
        format!(
            "required source manifest is unavailable at {}: {error}",
            path.display()
        )
    })?;
    let manifest: Value = serde_json::from_slice(&bytes)?;
    validate_value(&manifest, &bin_dir)?;

    let mut report = manifest;
    report["manifest_sha256"] = json!(sha256(&bytes));
    Ok(report)
}

fn validate_value(manifest: &Value, bin_dir: &Path) -> Result<(), Box<dyn Error>> {
    require_keys(
        manifest,
        &[
            "schema",
            "source_commit",
            "source_tree",
            "source_worktree",
            "source_clean",
            "build_log",
            "binary_sha256_file",
            "binaries",
        ],
        "source manifest",
    )?;
    if manifest["schema"] != SCHEMA {
        return Err(format!(
            "unsupported source manifest schema: {:?}",
            manifest["schema"]
        )
        .into());
    }
    let source_commit = required_string(manifest, "source_commit")?;
    let source_tree = required_string(manifest, "source_tree")?;
    require_hex(source_commit, 40, "source_commit")?;
    require_hex(source_tree, 40, "source_tree")?;
    if manifest["source_clean"] != true {
        return Err("source manifest does not attest to a clean source worktree".into());
    }

    let source_worktree =
        canonical_directory(Path::new(required_string(manifest, "source_worktree")?))?;
    let actual_commit = git(&source_worktree, &["rev-parse", "HEAD"])?;
    let actual_tree = git(&source_worktree, &["rev-parse", "HEAD^{tree}"])?;
    let actual_status = git(
        &source_worktree,
        &["status", "--porcelain", "--untracked-files=all"],
    )?;
    if actual_commit != source_commit || actual_tree != source_tree || !actual_status.is_empty() {
        return Err(format!(
            "source worktree does not match its manifest: commit={actual_commit}, tree={actual_tree}, status={actual_status:?}"
        )
        .into());
    }

    verify_evidence_file(manifest, "build_log")?;
    let build_log_path = evidence_path(manifest, "build_log")?;
    let build_log = fs::read_to_string(&build_log_path)?;
    if !build_log.contains("Finished `dev` profile")
        && !build_log.contains("Finished `release` profile")
    {
        return Err(format!(
            "build log has no successful dev or release completion: {}",
            build_log_path.display()
        )
        .into());
    }

    verify_evidence_file(manifest, "binary_sha256_file")?;
    let hashes_path = evidence_path(manifest, "binary_sha256_file")?;
    let built_hashes = parse_hash_file(&hashes_path)?;
    let binaries = manifest["binaries"]
        .as_array()
        .ok_or("source manifest binaries must be an array")?;
    if binaries.len() != REQUIRED_BINARIES.len() {
        return Err(format!(
            "source manifest must list exactly {} binaries, found {}",
            REQUIRED_BINARIES.len(),
            binaries.len()
        )
        .into());
    }

    let mut manifest_hashes = HashMap::new();
    for binary in binaries {
        require_keys(binary, &["name", "sha256"], "binary source entry")?;
        let name = required_string(binary, "name")?;
        let digest = required_string(binary, "sha256")?;
        require_hex(digest, 64, &format!("binary {name} sha256"))?;
        if manifest_hashes
            .insert(name.to_owned(), digest.to_owned())
            .is_some()
        {
            return Err(format!("duplicate binary source entry {name:?}").into());
        }
    }

    let expected_names = REQUIRED_BINARIES
        .iter()
        .map(|(name, _)| *name)
        .collect::<Vec<_>>();
    if manifest_hashes.len() != expected_names.len()
        || expected_names
            .iter()
            .any(|name| !manifest_hashes.contains_key(*name))
    {
        return Err(
            format!("source manifest binary names must be exactly {expected_names:?}").into(),
        );
    }

    if built_hashes.len() != HASH_EVIDENCE_NAMES.len()
        || HASH_EVIDENCE_NAMES
            .iter()
            .any(|name| !built_hashes.contains_key(*name))
    {
        return Err(
            format!("binary hash evidence must list exactly {HASH_EVIDENCE_NAMES:?}").into(),
        );
    }

    for (name, build_name) in REQUIRED_BINARIES {
        let staged_path = bin_dir.join(name);
        if !staged_path.is_file() {
            return Err(format!(
                "required staged binary is missing: {}",
                staged_path.display()
            )
            .into());
        }
        let staged_hash = sha256_file(&staged_path)?;
        let declared_hash = manifest_hashes
            .get(name)
            .ok_or_else(|| format!("source manifest is missing binary {name:?}"))?;
        let built_hash = built_hashes
            .get(build_name)
            .ok_or_else(|| format!("build hash evidence is missing {build_name:?}"))?;
        let staged_evidence_hash = built_hashes
            .get(name)
            .ok_or_else(|| format!("staged hash evidence is missing {name:?}"))?;
        if &staged_hash != declared_hash
            || &staged_hash != built_hash
            || &staged_hash != staged_evidence_hash
        {
            return Err(format!(
                "binary hash mismatch for {name}: manifest={declared_hash}, build={built_hash}, staged_evidence={staged_evidence_hash}, staged={staged_hash}"
            )
            .into());
        }
    }
    Ok(())
}

fn verify_evidence_file(manifest: &Value, key: &str) -> Result<(), Box<dyn Error>> {
    require_keys(&manifest[key], &["path", "sha256"], key)?;
    let path = evidence_path(manifest, key)?;
    let expected = required_string(&manifest[key], "sha256")?;
    require_hex(expected, 64, &format!("{key} sha256"))?;
    let actual = sha256_file(&path)?;
    if expected != actual {
        return Err(
            format!("{key} evidence digest changed: expected {expected}, got {actual}").into(),
        );
    }
    Ok(())
}

fn evidence_path(manifest: &Value, key: &str) -> Result<PathBuf, Box<dyn Error>> {
    let raw = required_string(&manifest[key], "path")?;
    let path = Path::new(raw);
    if !path.is_absolute() {
        return Err(format!("{key} path must be absolute: {raw}").into());
    }
    canonical_file(path)
}

fn parse_hash_file(path: &Path) -> Result<HashMap<String, String>, Box<dyn Error>> {
    let contents = fs::read_to_string(path)?;
    let mut hashes = HashMap::new();
    for (line_index, line) in contents.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        if line.len() < 66 {
            return Err(format!(
                "invalid SHA-256 evidence line {} in {}",
                line_index + 1,
                path.display()
            )
            .into());
        }
        if !line.is_char_boundary(64) {
            return Err(format!(
                "invalid SHA-256 evidence line {} in {}",
                line_index + 1,
                path.display()
            )
            .into());
        }
        let digest = &line[..64];
        require_hex(digest, 64, "binary SHA-256 evidence")?;
        let raw_path = line[64..].trim();
        let raw_path = raw_path.strip_prefix('*').unwrap_or(raw_path);
        if raw_path.is_empty() {
            return Err(format!(
                "binary path is missing from evidence line {}",
                line_index + 1
            )
            .into());
        }
        let name = Path::new(raw_path)
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| format!("invalid binary path on evidence line {}", line_index + 1))?;
        if let Some(previous) = hashes.insert(name.to_owned(), digest.to_owned())
            && previous != digest
        {
            return Err(format!(
                "binary hash evidence differs for repeated name {name:?}: {previous} and {digest}"
            )
            .into());
        }
    }
    Ok(hashes)
}

fn require_keys(value: &Value, keys: &[&str], name: &str) -> Result<(), Box<dyn Error>> {
    let object = value
        .as_object()
        .ok_or_else(|| format!("{name} must be an object"))?;
    if object.len() != keys.len() || keys.iter().any(|key| !object.contains_key(*key)) {
        return Err(format!("{name} must contain exactly these keys: {keys:?}").into());
    }
    Ok(())
}

fn required_string<'a>(value: &'a Value, key: &str) -> Result<&'a str, Box<dyn Error>> {
    value[key]
        .as_str()
        .ok_or_else(|| format!("source manifest field {key:?} must be a string").into())
}

fn require_hex(value: &str, length: usize, name: &str) -> Result<(), Box<dyn Error>> {
    if value.len() != length
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(format!("{name} must be {length} lowercase hexadecimal digits").into());
    }
    Ok(())
}

fn canonical_directory(path: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let path = path.canonicalize()?;
    if !path.is_dir() {
        return Err(format!("path is not a directory: {}", path.display()).into());
    }
    Ok(path)
}

fn canonical_file(path: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let path = path.canonicalize()?;
    if !path.is_file() {
        return Err(format!("path is not a file: {}", path.display()).into());
    }
    Ok(path)
}

fn git(directory: &Path, args: &[&str]) -> Result<String, Box<dyn Error>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(directory)
        .args(args)
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "git {:?} failed in {}: {}",
            args,
            directory.display(),
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}

fn sha256_file(path: &Path) -> Result<String, Box<dyn Error>> {
    Ok(sha256(&fs::read(path)?))
}

fn sha256(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
