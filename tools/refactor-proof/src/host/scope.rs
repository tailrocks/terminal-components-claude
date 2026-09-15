//! Candidate scope and path safety validation for freeze.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use super::authority::TrustedOverlay;
use super::git::{file_digest, hardlink_count, is_symlink, walk_worktree, GitCommand};
use crate::json_util::sha256_bytes;

#[derive(Debug, Clone)]
pub(super) struct VerifyScope {
    pub writable_paths: Vec<String>,
    pub forbidden_paths: Vec<String>,
}

pub(super) fn load_verify_scope(verify_toml: &Path) -> Result<VerifyScope, String> {
    let text = std::fs::read_to_string(verify_toml).map_err(|error| error.to_string())?;
    Ok(VerifyScope {
        writable_paths: parse_toml_string_array(&text, "writable_paths")?,
        forbidden_paths: parse_toml_string_array(&text, "forbidden_paths")?,
    })
}

fn parse_toml_string_array(text: &str, key: &str) -> Result<Vec<String>, String> {
    let marker = format!("{key} =");
    let line = text
        .lines()
        .find(|line| line.trim_start().starts_with(&marker))
        .ok_or_else(|| format!("missing {key} in verify.toml"))?;
    let values = line
        .split('=')
        .nth(1)
        .ok_or_else(|| format!("invalid {key} line"))?
        .trim();
    if !values.starts_with('[') || !values.ends_with(']') {
        return Err(format!("{key} must be a TOML string array"));
    }
    let inner = values.trim_start_matches('[').trim_end_matches(']');
    inner
        .split(',')
        .map(|entry| {
            let trimmed = entry.trim();
            trimmed
                .strip_prefix('"')
                .and_then(|value| value.strip_suffix('"'))
                .map(str::to_owned)
                .ok_or_else(|| format!("invalid string entry in {key}"))
        })
        .collect()
}

pub(super) fn path_matches_any(path: &str, patterns: &[String]) -> bool {
    patterns.iter().any(|pattern| path_matches_pattern(path, pattern))
}

pub(super) fn path_matches_pattern(path: &str, pattern: &str) -> bool {
    if let Some(prefix) = pattern.strip_suffix("/**") {
        path == prefix || path.starts_with(&format!("{prefix}/"))
    } else {
        path == pattern
    }
}

pub(super) fn validate_candidate(
    candidate: &Path,
    scope_base: &str,
    overlay: &TrustedOverlay,
    verify_scope: &VerifyScope,
) -> Result<(), &'static str> {
    let git = GitCommand::new(candidate);
    if git.rev_parse("HEAD").map_err(|_| "integrity")? != scope_base {
        return Err("integrity");
    }
    validate_unsafe_git(&git)?;
    validate_unsafe_paths(candidate)?;
    validate_scope(candidate, &git, scope_base, overlay, verify_scope)
}

fn validate_unsafe_git(git: &GitCommand) -> Result<(), &'static str> {
    for key in [
        "user.name",
        "user.email",
        "core.excludesfile",
        "core.worktree",
        "core.hooksPath",
    ] {
        let output = git
            .run_output(&["config", "--local", "--get", key])
            .map_err(|_| "integrity")?;
        if output.status.success() {
            return Err("unsafe-git");
        }
    }
    for (_path, marker) in git.ls_files_verbose().map_err(|_| "integrity")? {
        if matches!(marker, 'h' | 'S') {
            return Err("unsafe-git");
        }
    }
    for (_path, mode) in git.ls_files_stage().map_err(|_| "integrity")? {
        if mode == "160000" {
            return Err("unsafe-git");
        }
    }
    Ok(())
}

fn validate_unsafe_paths(candidate: &Path) -> Result<(), &'static str> {
    for relative in walk_worktree(candidate).map_err(|_| "integrity")? {
        let path = candidate.join(&relative);
        if is_symlink(&path) {
            return Err("unsafe-path");
        }
        if hardlink_count(&path).map_err(|_| "integrity")? > 1 {
            return Err("unsafe-path");
        }
    }
    Ok(())
}

fn validate_scope(
    candidate: &Path,
    git: &GitCommand,
    scope_base: &str,
    overlay: &TrustedOverlay,
    verify_scope: &VerifyScope,
) -> Result<(), &'static str> {
    let baseline_tree = git
        .rev_parse(&format!("{scope_base}^{{tree}}"))
        .map_err(|_| "integrity")?;
    let baseline_entries = git.ls_tree(&baseline_tree).map_err(|_| "integrity")?;
    let mut baseline_blobs = HashMap::new();
    for (path, meta) in baseline_entries {
        let object = meta
            .split_whitespace()
            .nth(2)
            .ok_or("integrity")?;
        baseline_blobs.insert(path, object.to_owned());
    }

    for (path, expected_digest) in &overlay.paths {
        let actual = candidate.join(path);
        if !actual.is_file() {
            return Err("scope");
        }
        if file_digest(&actual).map_err(|_| "integrity")? != *expected_digest {
            return Err("scope");
        }
    }

    let mut present = HashSet::new();
    for relative in walk_worktree(candidate).map_err(|_| "integrity")? {
        let rel = relative.to_string_lossy().replace('\\', "/");
        present.insert(rel.clone());
        let path = candidate.join(&relative);
        if git.check_ignore(&rel).map_err(|_| "integrity")? {
            return Err("unsafe-git");
        }
        let digest = file_digest(&path).map_err(|_| "integrity")?;
        if path_matches_any(&rel, &verify_scope.forbidden_paths) {
            if let Some(object) = baseline_blobs.get(&rel) {
                let baseline = git.cat_file_blob(object).map_err(|_| "integrity")?;
                if sha256_bytes(&baseline) != digest {
                    return Err("scope");
                }
            } else if baseline_blobs.contains_key(&rel) {
                return Err("scope");
            }
        }
        match baseline_blobs.get(&rel) {
            Some(object) => {
                let baseline = git.cat_file_blob(object).map_err(|_| "integrity")?;
                if sha256_bytes(&baseline) != digest
                    && !path_matches_any(&rel, &verify_scope.writable_paths)
                {
                    return Err("scope");
                }
            }
            None => {
                if !path_matches_any(&rel, &verify_scope.writable_paths) {
                    return Err("scope");
                }
            }
        }
    }

    for path in baseline_blobs.keys() {
        if !present.contains(path) && !path_matches_any(path, &verify_scope.writable_paths) {
            return Err("scope");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glob_patterns_match_expected_paths() {
        let patterns = vec!["src/**".to_owned()];
        assert!(path_matches_any("src/payload.txt", &patterns));
        assert!(path_matches_any("src/nested/x.txt", &patterns));
        assert!(!path_matches_any("outside.txt", &patterns));
        let forbidden = vec!["protected.txt".to_owned(), ".proof/**".to_owned()];
        assert!(path_matches_any("protected.txt", &forbidden));
        assert!(path_matches_any(".proof/check.py", &forbidden));
    }
}
