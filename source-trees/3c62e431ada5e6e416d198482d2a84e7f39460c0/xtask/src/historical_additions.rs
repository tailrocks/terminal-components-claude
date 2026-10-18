//! Exact reviewed recovery of absent historical HTML/PNG files; never a bless.
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
    process::Command,
};

const MANIFEST: &str = "tools/historical-render/pins.json";
const MANIFEST_SHA256: &str = "e2dcdb2952e5cbd283b841ce055962c4d961dfa967d43c879bdbc49a5abdb6cb";
const GENERATOR_COMMIT: &str = "c051c8fa61c2fd67eb06e122dd91fd2553c8c810";
const ARCHIVE_COMMIT: &str = "c12cad8728755cd2d03eefdd8e02891143fca86d";
const REVIEW: &str = "docs/historical-regeneration/independent-review.md";
const REVIEW_SHA256: &str = "c3898d76eea11f22051117788b5d6872d568227ce5619ddf63eaa91f0d9031dd";

fn verified(bytes: &[u8], expected: &str, label: &str) -> Result<(), String> {
    if format!("{:x}", Sha256::digest(bytes)) != expected {
        return Err(format!("historical addition: hash mismatch for {label}"));
    }
    Ok(())
}

fn regular_bytes(root: &Path, relative: &str) -> Result<Vec<u8>, String> {
    let path = root.join(relative);
    let metadata = fs::symlink_metadata(&path)
        .map_err(|error| format!("historical addition: cannot inspect {relative}: {error}"))?;
    if !metadata.file_type().is_file() {
        return Err(format!(
            "historical addition: {relative} is not a regular file"
        ));
    }
    fs::read(path).map_err(|error| format!("historical addition: cannot read {relative}: {error}"))
}

fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|error| format!("historical addition: git failed: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "historical addition: git {args:?} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(output.stdout)
}

struct Inventory {
    inputs: BTreeMap<String, String>,
    outputs: BTreeMap<String, String>,
}

fn inventory(bytes: &[u8]) -> Result<Inventory, String> {
    // The digest fixes the whole reviewed map, not a caller-editable allowlist.
    verified(bytes, MANIFEST_SHA256, MANIFEST)?;
    let value: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|error| format!("historical addition: malformed manifest: {error}"))?;
    let captures = value["captures"]
        .as_array()
        .ok_or("historical addition: missing captures")?;
    if captures.len() != 499 {
        return Err("historical addition: expected exactly 499 captures".to_owned());
    }
    let mut inputs = BTreeMap::new();
    let mut outputs = BTreeMap::new();
    for capture in captures {
        let id = capture["recipe_id"]
            .as_str()
            .ok_or("historical addition: missing recipe ID")?;
        if id.is_empty() || !id.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_') {
            return Err("historical addition: unsafe recipe ID".to_owned());
        }
        for extension in ["ansi", "cursor", "txt", "html", "png"] {
            let output = matches!(extension, "html" | "png");
            let item = &capture[if output { "outputs" } else { "inputs" }][extension];
            let path = format!("baseline/before/{id}.{extension}");
            let expected_name = if output {
                format!("{id}.{extension}")
            } else {
                path.clone()
            };
            if item[if output { "name" } else { "path" }].as_str() != Some(&expected_name) {
                return Err(format!(
                    "historical addition: unexpected path for {id}.{extension}"
                ));
            }
            let hash = item["sha256"]
                .as_str()
                .ok_or("historical addition: missing hash")?;
            if hash.len() != 64 || !hash.bytes().all(|c| c.is_ascii_hexdigit()) {
                return Err(format!("historical addition: invalid hash for {path}"));
            }
            if (if output { &mut outputs } else { &mut inputs })
                .insert(path, hash.to_owned())
                .is_some()
            {
                return Err("historical addition: duplicate artifact".to_owned());
            }
        }
    }
    Ok(Inventory { inputs, outputs })
}

fn authorize_outputs(
    outputs: &BTreeMap<String, String>,
    base_paths: &BTreeSet<String>,
    mut read: impl FnMut(&str) -> Result<Vec<u8>, String>,
) -> Result<BTreeSet<String>, String> {
    let mut allowed = BTreeSet::new();
    for (path, hash) in outputs {
        verified(&read(path)?, hash, path)?;
        if !base_paths.contains(path) {
            allowed.insert(path.clone());
        }
    }
    Ok(allowed)
}

/// Return only reviewed paths absent in the comparison base. Validate the full
/// installation even when Git reports no change, so missing files cannot vanish
/// from discovery and an ignored/untracked partial installation cannot pass.
pub(crate) fn reviewed_additions(root: &Path, base: &str) -> Result<BTreeSet<String>, String> {
    git(root, &["merge-base", "--is-ancestor", ARCHIVE_COMMIT, base])?;
    let pinned = git(root, &["show", &format!("{GENERATOR_COMMIT}:{MANIFEST}")])?;
    verified(&pinned, MANIFEST_SHA256, "committed generator manifest")?;
    let inventory = inventory(&regular_bytes(root, MANIFEST)?)?;
    verified(&regular_bytes(root, REVIEW)?, REVIEW_SHA256, REVIEW)?;
    for (path, hash) in [
        (
            "docs/historical-regeneration/FONT-OFL.txt",
            "30f0c136e3c88e422d0791acd97238870f9054a9729bc34cf2ff0d4ed8cac4ad",
        ),
        (
            "docs/historical-regeneration/PROVENANCE.json",
            "001f4cf50addaff6c59a3500416c0b681904f7db0f1fd9436990c397746e1dfc",
        ),
    ] {
        verified(&regular_bytes(root, path)?, hash, path)?;
    }
    let archive = fs::symlink_metadata(root.join("baseline/before"))
        .map_err(|error| format!("historical addition: missing archive: {error}"))?;
    if !archive.file_type().is_dir() {
        return Err("historical addition: archive must be a real directory".to_owned());
    }
    for entry in walkdir::WalkDir::new(root.join("baseline/before")).follow_links(false) {
        let entry =
            entry.map_err(|error| format!("historical addition: cannot read entry: {error}"))?;
        let relative = entry
            .path()
            .strip_prefix(root)
            .map_err(|error| format!("historical addition: cannot relativize entry: {error}"))?
            .to_string_lossy();
        if (relative.ends_with(".png") || relative.ends_with(".html"))
            && !inventory.outputs.contains_key(relative.as_ref())
        {
            return Err(format!("historical addition: unreviewed output {relative}"));
        }
    }
    for (path, hash) in &inventory.inputs {
        verified(&regular_bytes(root, path)?, hash, path)?;
    }
    // ls-tree distinguishes absence from empty blobs and Git failures. Never
    // infer absence from `git show` returning empty text or a failing command.
    let names = git(
        root,
        &[
            "ls-tree",
            "-r",
            "--name-only",
            "-z",
            base,
            "--",
            "baseline/before",
        ],
    )?;
    let base_paths = names
        .split(|c| *c == 0)
        .filter(|name| !name.is_empty())
        .map(|name| {
            String::from_utf8(name.to_vec())
                .map_err(|_| "historical addition: non-UTF8 base path".to_owned())
        })
        .collect::<Result<BTreeSet<_>, _>>()?;
    authorize_outputs(&inventory.outputs, &base_paths, |path| {
        regular_bytes(root, path)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> BTreeMap<String, String> {
        BTreeMap::from([(
            "baseline/before/one.png".to_owned(),
            format!("{:x}", Sha256::digest(b"reviewed")),
        )])
    }

    #[test]
    fn exact_absent_artifact_is_the_only_addition_exception() {
        let allowed =
            authorize_outputs(&sample(), &BTreeSet::new(), |_| Ok(b"reviewed".to_vec())).unwrap();
        assert_eq!(
            allowed,
            BTreeSet::from(["baseline/before/one.png".to_owned()])
        );
        assert!(!allowed.contains("baseline/before/unreviewed.png"));
    }

    #[test]
    fn existing_base_blob_never_receives_an_exception_even_if_bytes_match() {
        let existing = BTreeSet::from(["baseline/before/one.png".to_owned()]);
        assert!(
            authorize_outputs(&sample(), &existing, |_| Ok(b"reviewed".to_vec()))
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn tampered_and_missing_artifacts_fail_closed() {
        assert!(
            authorize_outputs(&sample(), &BTreeSet::new(), |_| Ok(b"mutation".to_vec())).is_err()
        );
        assert!(
            authorize_outputs(&sample(), &BTreeSet::new(), |_| Err("missing".to_owned())).is_err()
        );
    }

    #[test]
    fn malformed_and_tampered_manifests_cannot_change_the_reviewed_map() {
        assert!(inventory(b"{").is_err());
        let bytes = include_bytes!("../../tools/historical-render/pins.json");
        let inventory = inventory(bytes).unwrap();
        assert_eq!(inventory.inputs.len(), 1497);
        assert_eq!(inventory.outputs.len(), 998);
        let mut mutation = bytes.to_vec();
        mutation.push(b' ');
        assert!(super::inventory(&mutation).is_err());
    }

    #[test]
    fn invalid_or_unrelated_base_cannot_authorize_recovery() {
        assert!(reviewed_additions(&crate::root(), "not-a-revision").is_err());
        assert!(
            reviewed_additions(&crate::root(), "c12cad8728755cd2d03eefdd8e02891143fca86d^")
                .is_err()
        );
    }
}
