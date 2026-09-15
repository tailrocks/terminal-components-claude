//! Host-owned Git helpers with executor configuration disabled.

use std::collections::HashMap;
use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use crate::json_util::sha256_bytes;

pub struct GitCommand {
    repository: PathBuf,
}

impl GitCommand {
    pub fn new(repository: impl Into<PathBuf>) -> Self {
        Self {
            repository: repository.into(),
        }
    }

    pub fn run(&self, args: &[&str]) -> Result<String, String> {
        let output = self.run_output(args)?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
        } else {
            Err(format!(
                "git {} failed: {}",
                args.join(" "),
                String::from_utf8_lossy(&output.stderr).trim()
            ))
        }
    }

    pub fn run_output(&self, args: &[&str]) -> Result<Output, String> {
        Command::new("git")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_COUNT", "3")
            .env("GIT_CONFIG_KEY_0", "core.hooksPath")
            .env("GIT_CONFIG_VALUE_0", "/dev/null")
            .env("GIT_CONFIG_KEY_1", "user.name")
            .env("GIT_CONFIG_VALUE_1", "tc-proof-host")
            .env("GIT_CONFIG_KEY_2", "user.email")
            .env("GIT_CONFIG_VALUE_2", "host@example.invalid")
            .arg("-C")
            .arg(&self.repository)
            .args(args)
            .output()
            .map_err(|error| error.to_string())
    }

    pub fn rev_parse(&self, reference: &str) -> Result<String, String> {
        self.run(&["rev-parse", reference])
    }

    pub fn commit_tree(&self, tree: &str, parent: &str, message: &str) -> Result<String, String> {
        self.run(&["commit-tree", tree, "-p", parent, "-m", message])
    }

    pub fn update_ref(&self, reference: &str, new_value: &str, old_value: &str) -> Result<(), String> {
        let output = self
            .run_output(&["update-ref", reference, new_value, old_value])
            .map_err(|error| error.to_string())?;
        if output.status.success() {
            Ok(())
        } else {
            Err(format!(
                "git update-ref {reference} failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ))
        }
    }

    pub fn ls_tree(&self, tree: &str) -> Result<HashMap<String, String>, String> {
        let output = self.run(&["ls-tree", "-r", tree])?;
        let mut entries = HashMap::new();
        for line in output.lines() {
            let Some((meta, path)) = line.split_once('\t') else {
                continue;
            };
            let mode = meta.split_whitespace().next().unwrap_or_default();
            if mode == "160000" {
                return Err("submodule entry detected".to_string());
            }
            entries.insert(path.to_owned(), meta.to_owned());
        }
        Ok(entries)
    }

    pub fn blob_hash(&self, object: &str) -> Result<String, String> {
        self.run(&["rev-parse", &format!("{object}^{{blob}}")])
    }

    pub fn cat_file_blob(&self, object: &str) -> Result<Vec<u8>, String> {
        let output = self
            .run_output(&["cat-file", "blob", object])
            .map_err(|error| error.to_string())?;
        if output.status.success() {
            Ok(output.stdout)
        } else {
            Err(format!(
                "git cat-file blob {object} failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ))
        }
    }

    pub fn ls_files_verbose(&self) -> Result<Vec<(String, char)>, String> {
        let output = self.run(&["ls-files", "-v"])?;
        Ok(output
            .lines()
            .filter_map(|line| {
                let mut parts = line.split_whitespace();
                let flag = parts.next()?;
                let path = parts.next()?;
                flag.chars().next().map(|marker| (path.to_owned(), marker))
            })
            .collect())
    }

    pub fn ls_files_stage(&self) -> Result<Vec<(String, String)>, String> {
        let output = self.run(&["ls-files", "-s"])?;
        Ok(output
            .lines()
            .filter_map(|line| {
                let (meta, path) = line.split_once('\t')?;
                let mode = meta.split_whitespace().next()?;
                Some((path.to_owned(), mode.to_owned()))
            })
            .collect())
    }

    pub fn local_config_entries(&self) -> Result<Vec<String>, String> {
        let output = self.run_output(&["config", "--local", "--list"])?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout)
                .lines()
                .map(str::to_owned)
                .filter(|line| !line.is_empty())
                .collect())
        } else if output.status.code() == Some(128) {
            Ok(Vec::new())
        } else {
            Err(format!(
                "git config --local --list failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ))
        }
    }

    pub fn check_ignore(&self, path: &str) -> Result<bool, String> {
        let output = self
            .run_output(&["check-ignore", "-q", "--", path])
            .map_err(|error| error.to_string())?;
        Ok(output.status.success())
    }

    pub fn write_tree_from_worktree(
        &self,
        worktree: &Path,
        scratch_root: &Path,
    ) -> Result<String, String> {
        let build_dir = scratch_root.join("tree-build");
        if build_dir.exists() {
            fs::remove_dir_all(&build_dir).map_err(|error| error.to_string())?;
        }
        copy_worktree(worktree, &build_dir).map_err(|error| error.to_string())?;
        let git = GitCommand::new(&build_dir);
        git.run(&["init", "-q", "--initial-branch=fixture"])?;
        git.run(&["add", "-A"])?;
        git.run(&["write-tree"])
    }
}

fn copy_worktree(source: &Path, destination: &Path) -> io::Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_name = entry.file_name();
        if file_name == ".git" {
            continue;
        }
        let source_path = entry.path();
        let target_path = destination.join(file_name);
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            copy_worktree(&source_path, &target_path)?;
        } else if file_type.is_file() {
            fs::copy(source_path, target_path)?;
        }
    }
    Ok(())
}

pub fn file_digest(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|error| error.to_string())?;
    Ok(sha256_bytes(&bytes))
}

pub fn is_symlink(path: &Path) -> bool {
    path.symlink_metadata()
        .map(|meta| meta.file_type().is_symlink())
        .unwrap_or(false)
}

pub fn hardlink_count(path: &Path) -> Result<u64, String> {
    path.metadata()
        .map(|meta| meta.nlink())
        .map_err(|error| error.to_string())
}

pub fn resolve_within(base: &Path, path: &Path) -> Result<PathBuf, String> {
    let resolved = path
        .canonicalize()
        .or_else(|_| {
            let mut current = base.to_path_buf();
            for component in path.components() {
                current.push(component);
            }
            current.canonicalize()
        })
        .map_err(|error| error.to_string())?;
    let base_resolved = base
        .canonicalize()
        .map_err(|error| error.to_string())?;
    if resolved.starts_with(&base_resolved) {
        Ok(resolved)
    } else {
        Err("path escapes candidate root".to_string())
    }
}

pub fn walk_worktree(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut paths = Vec::new();
    walk_worktree_inner(root, root, &mut paths)?;
    paths.sort();
    Ok(paths)
}

fn walk_worktree_inner(root: &Path, current: &Path, paths: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(current).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        if path.file_name().is_some_and(|name| name == ".git") {
            continue;
        }
        let file_type = entry.file_type().map_err(|error| error.to_string())?;
        let relative = path.strip_prefix(root).map_err(|error| error.to_string())?.to_path_buf();
        if file_type.is_symlink() {
            paths.push(relative);
            continue;
        }
        if file_type.is_dir() {
            walk_worktree_inner(root, &path, paths)?;
        } else {
            paths.push(relative);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_tree_uses_worktree_not_index() {
        let temp = tempfile::tempdir().expect("tempdir");
        let repo = temp.path().join("repo");
        fs::create_dir_all(repo.join("src")).expect("mkdir");
        fs::write(repo.join("src/payload.txt"), b"pending\n").expect("write");
        fs::write(repo.join("src/obsolete.txt"), b"gone\n").expect("write");
        let git = GitCommand::new(&repo);
        git.run(&["init", "-q", "--initial-branch=fixture"])
            .expect("init");
        git.run(&["add", "-A"]).expect("add");
        git.run(&["commit", "-q", "-m", "base"]).expect("commit");
        fs::write(repo.join("src/payload.txt"), b"qualified\n").expect("write");
        fs::remove_file(repo.join("src/obsolete.txt")).expect("remove");
        fs::write(repo.join("src/new.txt"), b"new\n").expect("write");
        let index_tree = git.write_tree().expect("index tree");
        let worktree_tree = git
            .write_tree_from_worktree(&repo, temp.path())
            .expect("worktree tree");
        assert_ne!(index_tree, worktree_tree);
        let snapshot = tempfile::tempdir().expect("snapshot");
        copy_worktree(&repo, snapshot.path()).expect("copy");
        let snapshot_git = GitCommand::new(snapshot.path());
        snapshot_git
            .run(&["init", "-q", "--initial-branch=fixture"])
            .expect("init");
        snapshot_git.run(&["add", "-A"]).expect("add");
        assert_eq!(
            snapshot_git.run(&["write-tree"]).expect("snapshot tree"),
            worktree_tree
        );
    }
}

impl GitCommand {
    fn write_tree(&self) -> Result<String, String> {
        self.run(&["write-tree"])
    }
}
