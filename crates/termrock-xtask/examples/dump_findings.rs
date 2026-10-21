use std::path::PathBuf;
use termrock_xtask::ownership::check_workspace;

fn main() {
    let root = PathBuf::from(".");
    let exceptions = root.join("crates/termrock-xtask/exceptions/v1.json");
    let report = check_workspace(&root, &exceptions);
    let filter = std::env::args().nth(1).unwrap_or_default();
    for f in &report.findings {
        if filter.is_empty() || f.file.contains(&filter) {
            println!("{}:{} {} [{}] (in {})", f.file, f.line, f.rule, f.detail, f.symbol.clone().unwrap_or_default());
        }
    }
}
