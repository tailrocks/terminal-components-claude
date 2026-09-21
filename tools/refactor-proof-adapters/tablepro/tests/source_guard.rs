//! Adapter source must drive CLI / `TableProApp` constructors, never `set_surface`.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::fs;
use std::path::Path;

fn rust_files(dir: &Path, out: &mut Vec<(std::path::PathBuf, String)>) {
    let entries =
        fs::read_dir(dir).unwrap_or_else(|error| panic!("read {}: {error}", dir.display()));
    for entry in entries {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, out);
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
        out.push((path, source));
    }
}

#[test]
fn adapter_never_calls_set_surface() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    rust_files(&root.join("src"), &mut files);
    assert!(!files.is_empty(), "adapter rust sources missing");
    for (path, source) in files {
        assert!(
            !source.contains("set_surface("),
            "{} calls set_surface instead of driving CLI/handlers",
            path.display()
        );
    }
}
