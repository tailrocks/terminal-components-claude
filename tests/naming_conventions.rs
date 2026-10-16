//! Naming conventions of the widget API (shared-components plan §1.9, S0):
//! a widget's output enum is `XAction`, a key-map command enum is `XCmd`.
//! The old `XEvent` output names and `XAction` key-map names must not
//! come back, or the two vocabularies blur again.

use std::fs;
use std::path::{Path, PathBuf};

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("read_dir") {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// `(file, line, identifier)` for every `<keyword> <Ident>` in `dir`.
fn declarations(dir: &str, keyword: &str) -> Vec<(String, usize, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    rust_files(&root.join(dir), &mut files);
    files.sort();
    let mut found = Vec::new();
    for file in files {
        let text = fs::read_to_string(&file).expect("read source");
        for (n, line) in text.lines().enumerate() {
            let line = line.trim_start();
            if line.starts_with("//") {
                continue;
            }
            let mut rest = line;
            while let Some(at) = rest.find(keyword) {
                let before = &rest[..at];
                let after = &rest[at + keyword.len()..];
                rest = after;
                if before
                    .chars()
                    .last()
                    .is_some_and(|c| c.is_alphanumeric() || c == '_')
                {
                    continue;
                }
                let ident: String = after
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                if !ident.is_empty() {
                    let rel = file.strip_prefix(root).unwrap_or(&file);
                    found.push((rel.display().to_string(), n + 1, ident));
                }
            }
        }
    }
    found
}

#[test]
fn key_map_command_enums_are_cmds() {
    let wrong: Vec<_> = declarations("src", "impl Actions for ")
        .into_iter()
        .filter(|(_, _, name)| !name.ends_with("Cmd"))
        .collect();
    assert!(
        wrong.is_empty(),
        "key-map command enums (`impl Actions`) are named `XCmd` (plan §1.9): {wrong:?}"
    );
}
