use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

fn main() {
    let root = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let mut paths = Vec::new();
    collect_files(&root, &root, &mut paths);
    paths.sort();

    let mut digest = Sha256::new();
    for relative in &paths {
        let path = root.join(relative);
        let bytes = fs::read(&path).unwrap_or_else(|error| {
            panic!("read suite input {}: {error}", path.display());
        });
        let relative_text = relative.to_string_lossy();
        digest.update((relative.as_os_str().len() as u64).to_be_bytes());
        digest.update(relative_text.as_bytes());
        digest.update((bytes.len() as u64).to_be_bytes());
        digest.update(bytes);
        println!("cargo:rerun-if-changed={}", path.display());
    }
    println!(
        "cargo:rustc-env=TERMROCK_E2E_COMPILED_SUITE_SHA256={:x}",
        digest.finalize()
    );
}

fn collect_files(root: &Path, current: &Path, paths: &mut Vec<PathBuf>) {
    println!("cargo:rerun-if-changed={}", current.display());
    let entries = fs::read_dir(current)
        .unwrap_or_else(|error| panic!("read suite directory {}: {error}", current.display()));
    for entry in entries {
        let entry = entry.unwrap_or_else(|error| panic!("read suite directory entry: {error}"));
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .unwrap_or_else(|error| panic!("suite path escaped manifest dir: {error}"));
        if relative
            .components()
            .any(|component| component.as_os_str() == "target" || component.as_os_str() == ".git")
        {
            continue;
        }
        let metadata = fs::symlink_metadata(&path)
            .unwrap_or_else(|error| panic!("inspect suite path {}: {error}", path.display()));
        if metadata.file_type().is_symlink() {
            panic!("suite digest refuses symlink {}", path.display());
        }
        if metadata.is_dir() {
            collect_files(root, &path, paths);
        } else if metadata.is_file() {
            paths.push(relative.to_path_buf());
        }
    }
}
