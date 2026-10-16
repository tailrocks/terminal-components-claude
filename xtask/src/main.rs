//! Repository tasks. `cargo xtask visual …` manages the visual store
//! (`snapshots/`) of the tuiscotti suite in `tests/visual_baseline/`:
//!
//! | Command | What it does |
//! |---|---|
//! | `list [<filterset>]` | every scenario (TSV: name, test, table, tier, pending/stored, approval), optionally narrowed by a nextest filterset |
//! | `accept <filterset>` | copy the reviewed actual `.ansi` of every matching scenario from `target/tuiscotti/actual/` into `snapshots/` (drops tuisnap's `.txt/.png/.html`) |
//! | `render <name>` | approved `.ansi` to `target/tuiscotti/render/<name>.{png,html}` for viewing |
//! | `import <dir> [--dry-run]` | copy a captured `actual/` tree (a CI `oracle-candidate`) into `snapshots/` with a change list per app/group |
//!
//! `accept` and `import` write the store, so they refuse to run when `CI`
//! is set: approval is explicit and local only (AGENTS.md, plan §B5). There
//! is no environment variable that blesses snapshots.
//!
//! The scenario inventory and the renderer come from the test binary itself
//! (`VISUAL_TOOL=list|render`, `tests/visual_baseline/tool.rs`); filtersets
//! are resolved by `cargo nextest list`, so they mean exactly what they mean
//! to `cargo nextest run`.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

const USAGE: &str = "\
usage: cargo xtask visual <command>

  list [<filterset>]          list scenarios (TSV), optionally narrowed by a nextest filterset
  accept <filterset>          approve reviewed actuals of the matching scenarios (local only)
  render <name>               render an approved scenario to target/tuiscotti/render/
  import <dir> [--dry-run]    import a captured actual/ tree into snapshots/ (local only)

filterset examples: 'test(/^tablepro::/)', 'test(/::connections::/) & test(/::120x40::/)',
                    'test(=showcase::pages::lists::72x20::nocolor)'";

/// tuisnap's per-scenario files; only `.ansi` survives an approval.
const LEGACY_EXTS: [&str; 3] = ["txt", "png", "html"];

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let result = match args.as_slice() {
        ["visual", "list"] => list(None),
        ["visual", "list", filter] => list(Some(filter)),
        ["visual", "accept", filter] => accept(filter),
        ["visual", "render", name] => render(name),
        ["visual", "import", dir] => import(Path::new(dir), false),
        ["visual", "import", dir, "--dry-run"] | ["visual", "import", "--dry-run", dir] => {
            import(Path::new(dir), true)
        }
        _ => Err(USAGE.to_string()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives in the repository root")
        .to_path_buf()
}

fn store() -> PathBuf {
    root().join("snapshots")
}

fn actual() -> PathBuf {
    root().join("target/tuiscotti/actual")
}

/// `accept` and `import` write the store: never in CI.
fn refuse_in_ci(what: &str) -> Result<(), String> {
    match std::env::var_os("CI") {
        Some(v) if !v.is_empty() => Err(format!(
            "refusing `{what}` because CI is set: approving snapshots is local only \
             (docs/visual-testing/README.md)"
        )),
        _ => Ok(()),
    }
}

fn cargo() -> Command {
    let mut cmd = Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()));
    cmd.current_dir(root());
    cmd
}

/// Build the visual test binary and return its path.
fn test_binary() -> Result<PathBuf, String> {
    let out = cargo()
        .args([
            "build",
            "--package",
            "junie-tui",
            "--test",
            "visual_baseline",
            "--message-format",
            "json-render-diagnostics",
        ])
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| format!("cargo build: {e}"))?;
    if !out.status.success() {
        return Err("cargo build --test visual_baseline failed".into());
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    stdout
        .lines()
        .filter(|l| l.contains(r#""reason":"compiler-artifact""#))
        .filter(|l| l.contains(r#""name":"visual_baseline""#))
        .find_map(|l| json_string_field(l, "executable"))
        .map(PathBuf::from)
        .ok_or_else(|| "no visual_baseline executable in cargo's output".into())
}

/// The value of a top-level `"field":"…"` in one line of cargo's JSON.
fn json_string_field(line: &str, field: &str) -> Option<String> {
    let start = line.find(&format!(r#""{field}":""#))? + field.len() + 4;
    let mut out = String::new();
    let mut chars = line[start..].chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => return Some(out),
            '\\' => match chars.next()? {
                'n' => out.push('\n'),
                't' => out.push('\t'),
                other => out.push(other),
            },
            c => out.push(c),
        }
    }
    None
}

/// One row of the test binary's `VISUAL_TOOL=list` output.
struct Row {
    name: String,
    test: String,
    table: String,
    tier: String,
    pending: String,
    approval: String,
}

fn inventory(binary: &Path) -> Result<Vec<Row>, String> {
    let out = Command::new(binary)
        .current_dir(root())
        .env("VISUAL_TOOL", "list")
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| format!("run {}: {e}", binary.display()))?;
    if !out.status.success() {
        return Err("VISUAL_TOOL=list failed".into());
    }
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|line| {
            let f: Vec<&str> = line.split('\t').collect();
            match f.as_slice() {
                [name, test, table, tier, pending, approval] => Ok(Row {
                    name: name.to_string(),
                    test: test.to_string(),
                    table: table.to_string(),
                    tier: tier.to_string(),
                    pending: pending.to_string(),
                    approval: approval.to_string(),
                }),
                _ => Err(format!("bad inventory line: {line:?}")),
            }
        })
        .collect()
}

/// Test names in the visual binary that `filter` selects, through
/// `cargo nextest list` (ignored tests included).
fn select(filter: &str) -> Result<BTreeSet<String>, String> {
    let out = cargo()
        .args([
            "nextest",
            "list",
            "--color",
            "never",
            "--run-ignored",
            "all",
            "-E",
            &format!("binary(visual_baseline) & ({filter})"),
        ])
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| format!("cargo nextest list: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "cargo nextest list rejected the filterset `{filter}`"
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|l| l.trim().strip_prefix("junie-tui::visual_baseline "))
        .map(str::to_string)
        .collect())
}

fn selected_rows(filter: Option<&str>) -> Result<Vec<Row>, String> {
    let rows = inventory(&test_binary()?)?;
    let Some(filter) = filter else {
        return Ok(rows);
    };
    let tests = select(filter)?;
    Ok(rows
        .into_iter()
        .filter(|r| tests.contains(&r.test))
        .collect())
}

fn group_of(name: &str) -> String {
    name.split('/').take(2).collect::<Vec<_>>().join("/")
}

fn list(filter: Option<&str>) -> Result<(), String> {
    let rows = selected_rows(filter)?;
    let mut summary: BTreeMap<(String, String), usize> = BTreeMap::new();
    println!("name\ttest\ttable\ttier\tstatus\tapproval");
    for r in &rows {
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}",
            r.name, r.test, r.table, r.tier, r.pending, r.approval
        );
        let app = r.name.split('/').next().unwrap_or("").to_string();
        *summary
            .entry((app, format!("{} {}", r.pending, r.approval)))
            .or_default() += 1;
    }
    eprintln!("{} scenario(s)", rows.len());
    for ((app, status), n) in summary {
        eprintln!("  {app:<9} {status:<20} {n}");
    }
    Ok(())
}

/// Write one approved `.ansi` and drop tuisnap's other files for it.
fn approve(name: &str, ansi: &[u8]) -> Result<(), String> {
    let target = store().join(format!("{name}.ansi"));
    std::fs::create_dir_all(target.parent().expect("nested name"))
        .map_err(|e| format!("mkdir for {}: {e}", target.display()))?;
    std::fs::write(&target, ansi).map_err(|e| format!("write {}: {e}", target.display()))?;
    for ext in LEGACY_EXTS {
        let legacy = store().join(format!("{name}.{ext}"));
        if legacy.exists() {
            std::fs::remove_file(&legacy)
                .map_err(|e| format!("remove {}: {e}", legacy.display()))?;
        }
    }
    Ok(())
}

fn accept(filter: &str) -> Result<(), String> {
    refuse_in_ci("accept")?;
    let rows = selected_rows(Some(filter))?;
    if rows.is_empty() {
        return Err(format!("the filterset `{filter}` selects no scenario"));
    }
    let mut accepted = 0usize;
    let mut unchanged = 0usize;
    let mut no_actual = Vec::new();
    for r in &rows {
        let source = actual().join(format!("{}.ansi", r.name));
        let Ok(ansi) = std::fs::read(&source) else {
            no_actual.push(r.name.as_str());
            continue;
        };
        let before = std::fs::read(store().join(format!("{}.ansi", r.name))).ok();
        if before.as_deref() == Some(ansi.as_slice()) && r.approval == "reapproved" {
            unchanged += 1;
            continue;
        }
        approve(&r.name, &ansi)?;
        println!("accepted {}", r.name);
        accepted += 1;
    }
    eprintln!(
        "{accepted} accepted, {unchanged} already approved, {} without an actual (not run, or skipped)",
        no_actual.len()
    );
    if accepted == 0 && unchanged == 0 {
        return Err(format!(
            "no actuals under {} for `{filter}`; run the scenarios first",
            actual().display()
        ));
    }
    Ok(())
}

fn render(name: &str) -> Result<(), String> {
    let binary = test_binary()?;
    let status = Command::new(&binary)
        .current_dir(root())
        .env("VISUAL_TOOL", "render")
        .env("VISUAL_NAME", name)
        .status()
        .map_err(|e| format!("run {}: {e}", binary.display()))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("render `{name}` failed"))
    }
}

fn walk_ansi(root: &Path, dir: &Path, out: &mut Vec<String>) -> Result<(), String> {
    let entries = std::fs::read_dir(dir).map_err(|e| format!("list {}: {e}", dir.display()))?;
    for entry in entries {
        let path = entry
            .map_err(|e| format!("list {}: {e}", dir.display()))?
            .path();
        if path.is_dir() {
            walk_ansi(root, &path, out)?;
        } else if path.extension().is_some_and(|e| e == "ansi") {
            let rel = path
                .strip_prefix(root)
                .expect("under root")
                .with_extension("");
            let rel: Vec<_> = rel
                .components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect();
            out.push(rel.join("/"));
        }
    }
    Ok(())
}

/// Cells only: an `.ansi` without its cursor trailer (tuisnap's approvals
/// carry none), for the change list against the legacy store.
fn cells(ansi: &[u8]) -> &[u8] {
    let text = ansi;
    let marker = b"\x1b[?25";
    match text.windows(marker.len()).rposition(|w| w == marker) {
        Some(i) => &text[..i],
        None => text,
    }
}

fn import(dir: &Path, dry_run: bool) -> Result<(), String> {
    if !dry_run {
        refuse_in_ci("import")?;
    }
    if !dir.is_dir() {
        return Err(format!("{} is not a directory", dir.display()));
    }
    let known: BTreeMap<String, Row> = inventory(&test_binary()?)?
        .into_iter()
        .map(|r| (r.name.clone(), r))
        .collect();
    let mut found = Vec::new();
    walk_ansi(dir, dir, &mut found)?;
    found.sort();
    // status per group: new, changed, cursor-only, unchanged
    let mut groups: BTreeMap<String, [usize; 4]> = BTreeMap::new();
    let mut unknown = Vec::new();
    for name in &found {
        if !known.contains_key(name) {
            unknown.push(name.as_str());
            continue;
        }
        let candidate = std::fs::read(dir.join(format!("{name}.ansi")))
            .map_err(|e| format!("read {name}: {e}"))?;
        let slot = match std::fs::read(store().join(format!("{name}.ansi"))) {
            Err(_) => 0,
            Ok(old) if old == candidate => 3,
            Ok(old) if cells(&old) == cells(&candidate) => 2,
            Ok(_) => 1,
        };
        groups.entry(group_of(name)).or_default()[slot] += 1;
        if slot != 3 && !dry_run {
            approve(name, &candidate)?;
        }
    }
    println!(
        "{:<28} {:>6} {:>8} {:>12} {:>10}",
        "group", "new", "changed", "cursor-only", "unchanged"
    );
    for (group, [new, changed, cursor, same]) in &groups {
        println!("{group:<28} {new:>6} {changed:>8} {cursor:>12} {same:>10}");
    }
    let missing = known
        .values()
        .filter(|r| r.pending == "stored" && !found.contains(&r.name))
        .count();
    eprintln!(
        "{} imported{}, {} not in the scenario tables, {missing} stored scenario(s) absent from {}",
        found.len() - unknown.len(),
        if dry_run {
            " (dry run, nothing written)"
        } else {
            ""
        },
        unknown.len(),
        dir.display()
    );
    for name in unknown.iter().take(20) {
        eprintln!("  unknown: {name}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cargo_json_executable_is_extracted() {
        let line = r#"{"reason":"compiler-artifact","target":{"name":"visual_baseline"},"executable":"/t/deps/visual_baseline-abc","fresh":true}"#;
        assert_eq!(
            json_string_field(line, "executable").as_deref(),
            Some("/t/deps/visual_baseline-abc")
        );
        assert_eq!(json_string_field(line, "missing"), None);
    }

    #[test]
    fn cells_drop_only_the_cursor_trailer() {
        assert_eq!(cells(b"ab\x1b[0m\ncd\x1b[?25l"), b"ab\x1b[0m\ncd");
        assert_eq!(cells(b"ab\x1b[?25h\x1b[2;3H\x1b[5 q"), b"ab");
        assert_eq!(cells(b"plain"), b"plain");
    }

    #[test]
    fn groups_are_app_and_group() {
        assert_eq!(
            group_of("tablepro/connections/default/120x40/truecolor"),
            "tablepro/connections"
        );
    }

    #[test]
    fn accept_and_import_refuse_under_ci() {
        // The process environment is shared by the test thread only in this
        // binary's own process (nextest runs each test in its own process).
        // SAFETY: single-threaded test process under nextest.
        unsafe { std::env::set_var("CI", "true") };
        assert!(refuse_in_ci("accept").is_err());
        unsafe { std::env::set_var("CI", "") };
        assert!(refuse_in_ci("accept").is_ok());
        unsafe { std::env::remove_var("CI") };
        assert!(refuse_in_ci("accept").is_ok());
    }
}
