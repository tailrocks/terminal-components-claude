//! Repository tasks. `cargo xtask visual …` manages the visual store of the
//! tuiscotti suite in `tests/visual_baseline/`: `snapshots/` until oracle
//! v0 is imported, then `baselines/oracle/` (with `ORACLE.toml`):
//!
//! | Command | What it does |
//! |---|---|
//! | `list [<filterset>]` | every scenario (TSV: name, test, table, tier, pending/stored, approval), optionally narrowed by a nextest filterset |
//! | `accept <filterset>` | copy the reviewed actual `.ansi` of every matching scenario from `target/tuiscotti/actual/` into the store (drops tuisnap's `.txt/.png/.html`; re-seals an active oracle) |
//! | `render <name>` | approved `.ansi` to `target/tuiscotti/render/<name>.{png,html}` for viewing |
//! | `import <dir> [--dry-run] [--oracle [--source <c>]]` | copy a captured `actual/` tree (a CI `oracle-candidate`) into the store with a change list per app/group; `--oracle` writes `baselines/oracle/` and seals it (oracle v0, plan §E2) |
//! | `seal [--source <c>]` | rewrite `ORACLE.toml` (counts, harness hash, tuiscotti pin, approver, date) |
//! | `determinism <filterset>` | capture the selection twice (pending included, review rendering off) and fail on any byte difference (plan T6) |
//! | `ratchet [--check] [<filterset>]` | plan §F1: listed passing scenarios must still pass; newly passing ones are recorded in `tests/visual_baseline/progress/main-passing.txt` |
//! | `sync-oracle --from <c> …` | plan §E3: install `baselines/oracle/` and the harness from an approved commit, hash-checked |
//! | `check-touched [--base <ref>] [--dry-run]` | pre-push guard: run the re-approved groups of every app the branch touched |
//!
//! Commands that write the store or the progress list (`accept`, `import`,
//! `seal`, `ratchet`, `sync-oracle`) refuse to run when `CI` is set:
//! approval is explicit and local only (AGENTS.md, plan §B5). There is no
//! environment variable that blesses snapshots.
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
  import <dir> [--dry-run] [--oracle [--source <commit>]]
                              import a captured actual/ tree into the store; --oracle writes
                              baselines/oracle/ and seals ORACLE.toml (oracle v0; local only)
  seal [--source <commit>]    rewrite ORACLE.toml for the oracle as it is (local only)
  determinism <filterset>     capture the selection twice and fail if any capture differs
  ratchet [--check] [<filterset>]
                              no listed passing scenario may fail; record newly passing ones in
                              tests/visual_baseline/progress/main-passing.txt (--check: CI, read-only)
  sync-oracle --from <commit> [--repo <path>] [--harness-dest <dir>] [--allow-interim]
                              install baselines/oracle/ and the harness from an approved commit
  check-touched [--base <ref>] [--dry-run]
                              pre-push guard: run the re-approved groups of the apps touched since
                              the merge base with <ref> (default origin/design/interaction-polish)

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
        ["visual", "determinism", filter] => determinism(filter),
        ["visual", "import", rest @ ..] => import_args(rest),
        ["visual", "seal"] => seal(None, None),
        ["visual", "seal", "--source", source] => seal(Some(source), None),
        ["visual", "ratchet"] => ratchet(false, None),
        ["visual", "ratchet", "--check"] => ratchet(true, None),
        ["visual", "ratchet", "--check", filter] => ratchet(true, Some(filter)),
        ["visual", "ratchet", filter] => ratchet(false, Some(filter)),
        ["visual", "sync-oracle", rest @ ..] => sync_oracle(rest),
        ["visual", "check-touched", rest @ ..] => check_touched(rest),
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

/// The approved store the gate reads: `baselines/oracle/` once a capture
/// was imported into it (`pending = false`), else the legacy `snapshots/`.
fn store() -> PathBuf {
    if oracle_active() {
        oracle_dir()
    } else {
        root().join("snapshots")
    }
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

/// Write one approved `.ansi` into `into` and drop tuisnap's other files
/// for it.
fn approve(into: &Path, name: &str, ansi: &[u8]) -> Result<(), String> {
    let target = into.join(format!("{name}.ansi"));
    std::fs::create_dir_all(target.parent().expect("nested name"))
        .map_err(|e| format!("mkdir for {}: {e}", target.display()))?;
    std::fs::write(&target, ansi).map_err(|e| format!("write {}: {e}", target.display()))?;
    for ext in LEGACY_EXTS {
        let legacy = into.join(format!("{name}.{ext}"));
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
    let into = store();
    for r in &rows {
        let source = actual().join(format!("{}.ansi", r.name));
        let Ok(ansi) = std::fs::read(&source) else {
            no_actual.push(r.name.as_str());
            continue;
        };
        let before = std::fs::read(into.join(format!("{}.ansi", r.name))).ok();
        if before.as_deref() == Some(ansi.as_slice()) && r.approval == "reapproved" {
            unchanged += 1;
            continue;
        }
        approve(&into, &r.name, &ansi)?;
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
    if accepted > 0 && oracle_active() {
        // the oracle's manifest follows every approval (counts, harness)
        seal(None, None)?;
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

/// `import <dir> [--dry-run] [--oracle [--source <commit>]]`.
fn import_args(args: &[&str]) -> Result<(), String> {
    let mut dir = None;
    let mut dry_run = false;
    let mut oracle = false;
    let mut source = None;
    let mut it = args.iter();
    while let Some(&arg) = it.next() {
        match arg {
            "--dry-run" => dry_run = true,
            "--oracle" => oracle = true,
            "--source" => source = Some(*it.next().ok_or("--source needs a commit")?),
            a if !a.starts_with("--") && dir.is_none() => dir = Some(a),
            other => return Err(format!("unknown import argument `{other}`\n\n{USAGE}")),
        }
    }
    let dir = dir.ok_or_else(|| format!("import needs a directory\n\n{USAGE}"))?;
    if source.is_some() && !oracle {
        return Err("--source only applies to --oracle".into());
    }
    import(
        Path::new(dir),
        dry_run,
        oracle.then(|| source.unwrap_or("HEAD")),
    )
}

/// Import a captured `actual/` tree. With `oracle = Some(source commit)`
/// every scenario is written into `baselines/oracle/` (the change list is
/// still against the current store) and the manifest is sealed at that
/// commit with `pending = false`: oracle v0 (plan §E2).
fn import(dir: &Path, dry_run: bool, oracle: Option<&str>) -> Result<(), String> {
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
    let current = store();
    let into = if oracle.is_some() {
        oracle_dir()
    } else {
        current.clone()
    };
    for name in &found {
        if !known.contains_key(name) {
            unknown.push(name.as_str());
            continue;
        }
        let candidate = std::fs::read(dir.join(format!("{name}.ansi")))
            .map_err(|e| format!("read {name}: {e}"))?;
        let slot = match std::fs::read(current.join(format!("{name}.ansi"))) {
            Err(_) => 0,
            Ok(old) if old == candidate => 3,
            Ok(old) if cells(&old) == cells(&candidate) => 2,
            Ok(_) => 1,
        };
        groups.entry(group_of(name)).or_default()[slot] += 1;
        if (slot != 3 || oracle.is_some()) && !dry_run {
            approve(&into, name, &candidate)?;
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
    if let Some(source) = oracle
        && !dry_run
    {
        seal(Some(source), None)?;
    }
    Ok(())
}

/// Run the visual scenarios matching `filter` (pending ones included,
/// review rendering off) after deleting their old actuals; returns the
/// fresh actual `.ansi` of every selected scenario that produced one. The
/// nextest exit status is ignored: a mismatch against a stale approval is
/// expected here, only the captured bytes matter.
fn capture_fresh(filter: &str, rows: &[Row]) -> Result<BTreeMap<String, Vec<u8>>, String> {
    for r in rows {
        let _ = std::fs::remove_file(actual().join(format!("{}.ansi", r.name)));
    }
    cargo()
        .args([
            "nextest",
            "run",
            "--run-ignored",
            "only",
            "--no-fail-fast",
            "--status-level",
            "fail",
            "--final-status-level",
            "none",
            "-E",
            &format!("binary(visual_baseline) & ({filter})"),
        ])
        .env("VISUAL_FULL", "1")
        .env("VISUAL_REVIEW", "0")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| format!("cargo nextest run: {e}"))?;
    Ok(rows
        .iter()
        .filter_map(|r| {
            std::fs::read(actual().join(format!("{}.ansi", r.name)))
                .ok()
                .map(|b| (r.name.clone(), b))
        })
        .collect())
}

/// `determinism <filterset>`: capture the selection twice and compare the
/// two `.ansi` sets byte for byte (plan T6). Fails on any difference.
fn determinism(filter: &str) -> Result<(), String> {
    let rows = selected_rows(Some(filter))?;
    if rows.is_empty() {
        return Err(format!("the filterset `{filter}` selects no scenario"));
    }
    eprintln!("run 1 of 2: {} scenario(s)", rows.len());
    let first = capture_fresh(filter, &rows)?;
    eprintln!("run 2 of 2");
    let second = capture_fresh(filter, &rows)?;
    let report = compare_runs(
        &rows.iter().map(|r| r.name.clone()).collect::<Vec<_>>(),
        &first,
        &second,
    );
    for name in &report.differ {
        println!("differs\t{name}");
    }
    for name in &report.missing {
        println!("no-capture\t{name}");
    }
    eprintln!(
        "{} identical, {} differ, {} without a capture in one or both runs (boot or wait failed)",
        report.identical,
        report.differ.len(),
        report.missing.len()
    );
    if report.differ.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{} scenario(s) are not deterministic",
            report.differ.len()
        ))
    }
}

#[derive(Debug, Default, PartialEq)]
struct RunComparison {
    identical: usize,
    differ: Vec<String>,
    missing: Vec<String>,
}

fn compare_runs(
    names: &[String],
    first: &BTreeMap<String, Vec<u8>>,
    second: &BTreeMap<String, Vec<u8>>,
) -> RunComparison {
    let mut out = RunComparison::default();
    for name in names {
        match (first.get(name), second.get(name)) {
            (Some(a), Some(b)) if a == b => out.identical += 1,
            (Some(_), Some(_)) => out.differ.push(name.clone()),
            _ => out.missing.push(name.clone()),
        }
    }
    out
}

// ------------------------------------------------------------- oracle --

const ORACLE: &str = "baselines/oracle";
const MANIFEST: &str = "ORACLE.toml";
const APPS: [&str; 4] = ["showcase", "tablepro", "jackin", "holla"];

fn oracle_dir() -> PathBuf {
    root().join(ORACLE)
}

/// One flat `key = value` of `ORACLE.toml`, unquoted. The test binary's
/// `oracle_integrity` validates the whole format; xtask only reads keys.
fn manifest_value(text: &str, key: &str) -> Option<String> {
    text.lines().find_map(|l| {
        let l = l.split('#').next()?.trim();
        let (k, v) = l.split_once('=')?;
        (k.trim() == key).then(|| v.trim().trim_matches('"').to_string())
    })
}

fn read_manifest() -> Option<String> {
    std::fs::read_to_string(oracle_dir().join(MANIFEST)).ok()
}

/// The oracle is the approved store once a capture was imported.
fn oracle_active() -> bool {
    read_manifest()
        .and_then(|t| manifest_value(&t, "pending"))
        .is_some_and(|v| v == "false")
}

fn command_line(cmd: &mut Command, what: &str) -> Result<String, String> {
    let out = cmd
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| format!("{what}: {e}"))?;
    if !out.status.success() {
        return Err(format!("{what} failed"));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn git(args: &[&str]) -> Result<String, String> {
    command_line(
        Command::new("git").current_dir(root()).args(args),
        &format!("git {}", args.join(" ")),
    )
}

/// `harness_sha256` of a harness directory, computed by the test binary
/// (`VISUAL_TOOL=harness-hash`) so there is one implementation.
fn harness_hash(dir: Option<&Path>) -> Result<String, String> {
    let binary = test_binary()?;
    let mut cmd = Command::new(&binary);
    cmd.current_dir(root()).env("VISUAL_TOOL", "harness-hash");
    if let Some(dir) = dir {
        cmd.env("VISUAL_DIR", dir);
    }
    command_line(&mut cmd, "VISUAL_TOOL=harness-hash")
}

fn pinned_tuiscotti_rev() -> Result<String, String> {
    let toml = std::fs::read_to_string(root().join("Cargo.toml"))
        .map_err(|e| format!("Cargo.toml: {e}"))?;
    toml.lines()
        .find(|l| l.trim_start().starts_with("tuiscotti ="))
        .and_then(|l| l.split("rev = \"").nth(1))
        .and_then(|r| r.split('"').next())
        .map(str::to_string)
        .ok_or_else(|| "Cargo.toml has no tuiscotti `rev =` pin".into())
}

struct ManifestOut {
    version: u64,
    pending: bool,
    source_commit: String,
    tuiscotti_rev: String,
    harness_sha256: String,
    approved_by: String,
    date: String,
    counts: BTreeMap<String, u64>,
}

fn render_manifest(m: &ManifestOut) -> String {
    let mut out = format!(
        "# Visual oracle manifest (docs/visual-testing/plan.md §B4, §E).\n\
         # Written by `cargo xtask visual import --oracle | accept | seal`;\n\
         # checked by the default-run test `oracle_integrity`.\n\
         version = {}\n\
         pending = {}\n\
         source_commit = \"{}\"\n\
         tuiscotti_rev = \"{}\"\n\
         harness_sha256 = \"{}\"\n\
         approved_by = \"{}\"\n\
         date = \"{}\"\n\
         \n\
         [scenarios]\n",
        m.version,
        m.pending,
        m.source_commit,
        m.tuiscotti_rev,
        m.harness_sha256,
        m.approved_by,
        m.date
    );
    for app in APPS {
        out.push_str(&format!(
            "{app} = {}\n",
            m.counts.get(app).copied().unwrap_or(0)
        ));
    }
    out
}

fn oracle_counts() -> Result<BTreeMap<String, u64>, String> {
    let mut names = Vec::new();
    walk_ansi(&oracle_dir(), &oracle_dir(), &mut names)?;
    let mut counts: BTreeMap<String, u64> = APPS.iter().map(|a| (a.to_string(), 0)).collect();
    for name in names {
        let app = name.split('/').next().unwrap_or_default().to_string();
        *counts.entry(app).or_default() += 1;
    }
    Ok(counts)
}

/// Rewrite `ORACLE.toml` for the store as it is now: counts, harness hash,
/// tuiscotti pin, approver and date; `source` / `version` when given.
fn seal(source: Option<&str>, version: Option<u64>) -> Result<(), String> {
    refuse_in_ci("seal")?;
    let old = read_manifest().ok_or_else(|| format!("{ORACLE}/{MANIFEST} is missing"))?;
    let keep = |k: &str| manifest_value(&old, k).unwrap_or_default();
    let m = ManifestOut {
        version: match version {
            Some(v) => v,
            None => keep("version").parse().unwrap_or(0),
        },
        pending: false,
        source_commit: match source {
            Some(s) => git(&["rev-parse", &format!("{s}^{{commit}}")])?,
            None => keep("source_commit"),
        },
        tuiscotti_rev: pinned_tuiscotti_rev()?,
        harness_sha256: harness_hash(None)?,
        approved_by: git(&["config", "user.name"])?,
        date: command_line(Command::new("date").args(["-u", "+%Y-%m-%d"]), "date")?,
        counts: oracle_counts()?,
    };
    if m.source_commit.is_empty() {
        return Err("no source commit: pass --source <commit>".into());
    }
    std::fs::write(oracle_dir().join(MANIFEST), render_manifest(&m))
        .map_err(|e| format!("write {MANIFEST}: {e}"))?;
    eprintln!(
        "sealed {ORACLE} v{} at {}: {:?}",
        m.version, m.source_commit, m.counts
    );
    Ok(())
}
// ------------------------------------------------------------ ratchet --

/// Scenarios that pass on a refactor branch (plan §F1). Not part of the
/// harness hash, so `main` can grow it without a sync.
const PROGRESS: &str = "tests/visual_baseline/progress/main-passing.txt";

fn read_progress() -> Result<BTreeSet<String>, String> {
    match std::fs::read_to_string(root().join(PROGRESS)) {
        Ok(text) => Ok(text
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(str::to_string)
            .collect()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(BTreeSet::new()),
        Err(e) => Err(format!("read {PROGRESS}: {e}")),
    }
}

/// `(regressed, newly passing)` for the selected scenarios: listed ones
/// that no longer pass, and passing ones not listed yet.
fn ratchet_verdict(
    selected: &BTreeSet<String>,
    listed: &BTreeSet<String>,
    passing: &BTreeSet<String>,
) -> (Vec<String>, Vec<String>) {
    let regressed = listed
        .intersection(selected)
        .filter(|n| !passing.contains(*n))
        .cloned()
        .collect();
    let new = passing.difference(listed).cloned().collect();
    (regressed, new)
}

/// `ratchet [--check] [<filterset>]`: capture the selection (default: every
/// scenario), count a scenario as passing when its capture equals the
/// approved `.ansi` byte for byte, and hold the progress list: a listed
/// scenario that fails is an error; newly passing ones are added (or, with
/// `--check`, which CI may run, reported as an error until added).
fn ratchet(check: bool, filter: Option<&str>) -> Result<(), String> {
    if !check {
        refuse_in_ci("ratchet")?;
    }
    let filter = filter.unwrap_or("all()");
    let rows = selected_rows(Some(filter))?;
    if rows.is_empty() {
        return Err(format!("the filterset `{filter}` selects no scenario"));
    }
    eprintln!("capturing {} scenario(s)", rows.len());
    let fresh = capture_fresh(filter, &rows)?;
    let approved = store();
    let selected: BTreeSet<String> = rows.iter().map(|r| r.name.clone()).collect();
    let passing: BTreeSet<String> = rows
        .iter()
        .filter(|r| {
            fresh.get(&r.name).is_some_and(|ansi| {
                std::fs::read(approved.join(format!("{}.ansi", r.name))).is_ok_and(|ok| &ok == ansi)
            })
        })
        .map(|r| r.name.clone())
        .collect();
    let listed = read_progress()?;
    let (regressed, new) = ratchet_verdict(&selected, &listed, &passing);
    let mut per_app: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    for name in &selected {
        let e = per_app
            .entry(name.split('/').next().unwrap_or_default().to_string())
            .or_default();
        e.1 += 1;
        if passing.contains(name) {
            e.0 += 1;
        }
    }
    for (app, (pass, total)) in &per_app {
        println!("{app:<9} {pass:>6} / {total:<6} passing");
    }
    for name in &regressed {
        println!("regressed\t{name}");
    }
    if !regressed.is_empty() {
        return Err(format!(
            "{} listed scenario(s) no longer pass (review target/tuiscotti/actual/)",
            regressed.len()
        ));
    }
    if check {
        for name in &new {
            println!("unlisted\t{name}");
        }
        return if new.is_empty() {
            Ok(())
        } else {
            Err(format!(
                "{} newly passing scenario(s) are not in {PROGRESS}: run `cargo xtask visual ratchet` locally",
                new.len()
            ))
        };
    }
    let all: BTreeSet<String> = listed.union(&passing).cloned().collect();
    let path = root().join(PROGRESS);
    std::fs::create_dir_all(path.parent().expect("nested")).map_err(|e| format!("mkdir: {e}"))?;
    let mut text = String::from(
        "# Scenarios that pass on this branch; written by `cargo xtask visual ratchet`.\n\
         # No listed scenario may fail (plan §F1).\n",
    );
    for name in &all {
        text.push_str(name);
        text.push('\n');
    }
    std::fs::write(&path, text).map_err(|e| format!("write {PROGRESS}: {e}"))?;
    eprintln!(
        "{} newly passing added; {PROGRESS} lists {}",
        new.len(),
        all.len()
    );
    Ok(())
}

// -------------------------------------------------------- sync-oracle --

fn copy_tree(from: &Path, to: &Path) -> Result<usize, String> {
    std::fs::create_dir_all(to).map_err(|e| format!("mkdir {}: {e}", to.display()))?;
    let mut n = 0;
    for entry in std::fs::read_dir(from).map_err(|e| format!("list {}: {e}", from.display()))? {
        let path = entry
            .map_err(|e| format!("list {}: {e}", from.display()))?
            .path();
        let target = to.join(path.file_name().expect("named"));
        if path.is_dir() {
            n += copy_tree(&path, &target)?;
        } else {
            std::fs::copy(&path, &target).map_err(|e| format!("copy {}: {e}", path.display()))?;
            n += 1;
        }
    }
    Ok(n)
}

fn remove_tree(path: &Path) -> Result<(), String> {
    match std::fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("remove {}: {e}", path.display())),
    }
}

struct SyncArgs<'a> {
    from: &'a str,
    repo: PathBuf,
    harness_dest: &'a str,
    allow_interim: bool,
}

fn parse_sync_args<'a>(args: &[&'a str]) -> Result<SyncArgs<'a>, String> {
    let mut out = SyncArgs {
        from: "",
        repo: root(),
        harness_dest: "tests/visual_baseline",
        allow_interim: false,
    };
    let mut it = args.iter();
    while let Some(&arg) = it.next() {
        let mut value = || it.next().copied().ok_or(format!("{arg} needs a value"));
        match arg {
            "--from" => out.from = value()?,
            "--repo" => out.repo = PathBuf::from(value()?),
            "--harness-dest" => out.harness_dest = value()?,
            "--allow-interim" => out.allow_interim = true,
            other => return Err(format!("unknown sync-oracle argument `{other}`\n\n{USAGE}")),
        }
    }
    if out.from.is_empty() {
        return Err(format!("sync-oracle needs --from <commit>\n\n{USAGE}"));
    }
    Ok(out)
}

/// `sync-oracle --from <commit>` (plan §E3): take `baselines/oracle/` and
/// the harness from an approved commit of the reference branch (`--repo`,
/// default this checkout) and install them here, the harness at
/// `--harness-dest` (keeping its `progress/`). Refuses a pending oracle,
/// one below v1 unless `--allow-interim`, and a harness whose hash differs
/// from the manifest.
fn sync_oracle(args: &[&str]) -> Result<(), String> {
    refuse_in_ci("sync-oracle")?;
    let a = parse_sync_args(args)?;
    let repo = a.repo.to_string_lossy().into_owned();
    let git_at = |args: &[&str]| -> Result<String, String> {
        command_line(
            Command::new("git").arg("-C").arg(&repo).args(args),
            &format!("git -C {repo} {}", args.join(" ")),
        )
    };
    let sha = git_at(&["rev-parse", &format!("{}^{{commit}}", a.from)])?;
    let manifest = git_at(&["show", &format!("{sha}:{ORACLE}/{MANIFEST}")])?;
    if manifest_value(&manifest, "pending").as_deref() != Some("false") {
        return Err(format!(
            "{ORACLE} at {sha} is pending: nothing approved to sync"
        ));
    }
    let version: u64 = manifest_value(&manifest, "version")
        .and_then(|v| v.parse().ok())
        .ok_or("ORACLE.toml has no integer version")?;
    if version < 1 && !a.allow_interim {
        return Err(format!(
            "{ORACLE} at {sha} is v{version}; main syncs the published oracle (v1+) only (--allow-interim overrides)"
        ));
    }
    let work = root().join("target/xtask/sync-oracle").join(&sha);
    remove_tree(&work)?;
    std::fs::create_dir_all(&work).map_err(|e| format!("mkdir {}: {e}", work.display()))?;
    let tar = work.join("export.tar");
    git_at(&[
        "archive",
        "--format=tar",
        "-o",
        &tar.to_string_lossy(),
        &sha,
        "--",
        ORACLE,
        "tests/visual_baseline",
    ])?;
    command_line(
        Command::new("tar")
            .arg("-xf")
            .arg(&tar)
            .arg("-C")
            .arg(&work),
        "tar -x",
    )?;
    let incoming = work.join("tests/visual_baseline");
    let hash = harness_hash(Some(&incoming))?;
    let expected = manifest_value(&manifest, "harness_sha256").unwrap_or_default();
    if hash != expected {
        return Err(format!(
            "harness at {sha} hashes to {hash}, but its ORACLE.toml says {expected}: the oracle was not sealed for that harness"
        ));
    }
    let oracle_files = copy_tree(&work.join(ORACLE), &work.join("oracle-staged"))?;
    remove_tree(&oracle_dir())?;
    copy_tree(&work.join("oracle-staged"), &oracle_dir())?;
    let dest = root().join(a.harness_dest);
    let progress = dest.join("progress");
    let kept = work.join("progress-kept");
    if progress.is_dir() {
        copy_tree(&progress, &kept)?;
    }
    remove_tree(&dest)?;
    let harness_files = copy_tree(&incoming, &dest)?;
    if kept.is_dir() {
        copy_tree(&kept, &progress)?;
    }
    println!(
        "synced {ORACLE} v{version} ({oracle_files} files) and the harness ({harness_files} files) into {} from {sha}",
        a.harness_dest
    );
    println!("next: cargo nextest run (oracle_integrity), then commit both directories in one PR");
    Ok(())
}

// ------------------------------------------------------- check-touched --

/// Apps whose captures a change to `path` can affect (`*` = all four).
fn affected_by(path: &str) -> Vec<&'static str> {
    const ALL: [&str; 4] = APPS;
    let bins = [
        ("src/bin/showcase/", "showcase"),
        ("src/bin/tablepro/", "tablepro"),
        ("src/bin/jackin_preview/", "jackin"),
        ("src/bin/holla/", "holla"),
    ];
    if let Some((_, app)) = bins.iter().find(|(p, _)| path.starts_with(p)) {
        return vec![app];
    }
    for store in ["snapshots/", "baselines/oracle/"] {
        if let Some(rest) = path.strip_prefix(store) {
            let app = rest.split('/').next().unwrap_or_default();
            return ALL.into_iter().filter(|a| *a == app).collect();
        }
    }
    let shared = path.starts_with("src/")
        || path.starts_with("tests/visual_baseline/")
        || matches!(path, "Cargo.toml" | "Cargo.lock" | "rust-toolchain.toml");
    if shared { ALL.to_vec() } else { Vec::new() }
}

/// nextest filterset for the re-approved groups of `apps`.
fn touched_filter(rows: &[Row], apps: &BTreeSet<&str>) -> Option<String> {
    let groups: BTreeSet<String> = rows
        .iter()
        .filter(|r| r.approval == "reapproved")
        .filter(|r| apps.contains(r.name.split('/').next().unwrap_or_default()))
        .map(|r| group_of(&r.name))
        .collect();
    if groups.is_empty() {
        return None;
    }
    Some(
        groups
            .iter()
            .map(|g| format!("test(/^{}::/)", g.replace('/', "::")))
            .collect::<Vec<_>>()
            .join(" | "),
    )
}

/// `check-touched [--base <ref>] [--dry-run]`: the pre-push guard for UI
/// changes. Maps every file changed since the merge base with `--base`
/// (default `origin/design/interaction-polish`; committed, staged,
/// unstaged and untracked) to the apps it can affect and runs their
/// re-approved scenario groups. A failure means an approval went stale:
/// review `target/tuiscotti/` and re-approve on purpose, or fix the change.
fn check_touched(args: &[&str]) -> Result<(), String> {
    let mut base = "origin/design/interaction-polish";
    let mut dry_run = false;
    let mut it = args.iter();
    while let Some(&arg) = it.next() {
        match arg {
            "--base" => base = it.next().copied().ok_or("--base needs a ref")?,
            "--dry-run" => dry_run = true,
            other => {
                return Err(format!(
                    "unknown check-touched argument `{other}`\n\n{USAGE}"
                ));
            }
        }
    }
    let merge_base = git(&["merge-base", base, "HEAD"])?;
    let mut changed: BTreeSet<String> = git(&["diff", "--name-only", &merge_base])?
        .lines()
        .map(str::to_string)
        .collect();
    changed.extend(
        git(&["ls-files", "--others", "--exclude-standard"])?
            .lines()
            .map(str::to_string),
    );
    let apps: BTreeSet<&str> = changed.iter().flat_map(|p| affected_by(p)).collect();
    eprintln!(
        "{} changed file(s) since {base} ({}); affected apps: {}",
        changed.len(),
        &merge_base[..merge_base.len().min(9)],
        if apps.is_empty() {
            "none".to_string()
        } else {
            apps.iter().copied().collect::<Vec<_>>().join(", ")
        }
    );
    if apps.is_empty() {
        return Ok(());
    }
    let rows = inventory(&test_binary()?)?;
    let Some(filter) = touched_filter(&rows, &apps) else {
        eprintln!("no re-approved scenario in the affected apps yet; nothing gates");
        return Ok(());
    };
    println!("{filter}");
    if dry_run {
        return Ok(());
    }
    let status = cargo()
        .args([
            "nextest",
            "run",
            "--run-ignored",
            "only",
            "--no-fail-fast",
            "-E",
            &format!("binary(visual_baseline) & ({filter})"),
        ])
        .status()
        .map_err(|e| format!("cargo nextest run: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(
            "re-approved captures changed: review target/tuiscotti/{actual,expected,diff} and \
             re-approve the intended ones (`cargo xtask visual accept`, a snapshot-only commit \
             naming the change), or fix the regression"
                .into(),
        )
    }
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

    fn row(name: &str, approval: &str) -> Row {
        Row {
            name: name.into(),
            test: name.replace('/', "::"),
            table: "SURFACES".into(),
            tier: "full".into(),
            pending: "stored".into(),
            approval: approval.into(),
        }
    }

    #[test]
    fn touched_files_map_to_apps() {
        assert_eq!(
            affected_by("src/bin/showcase/pages/chrome.rs"),
            ["showcase"]
        );
        assert_eq!(affected_by("src/bin/jackin_preview/app.rs"), ["jackin"]);
        assert_eq!(affected_by("src/widgets/menu.rs"), APPS.to_vec());
        assert_eq!(affected_by("src/theme.rs"), APPS.to_vec());
        assert_eq!(
            affected_by("tests/visual_baseline/support.rs"),
            APPS.to_vec()
        );
        assert_eq!(affected_by("Cargo.lock"), APPS.to_vec());
        assert_eq!(
            affected_by("snapshots/tablepro/connections/default/120x40/truecolor.ansi"),
            ["tablepro"]
        );
        assert_eq!(
            affected_by("baselines/oracle/ORACLE.toml"),
            Vec::<&str>::new()
        );
        assert!(affected_by("docs/visual-testing/README.md").is_empty());
        assert!(affected_by("xtask/src/main.rs").is_empty());
    }

    #[test]
    fn touched_filter_runs_reapproved_groups_of_touched_apps() {
        let rows = [
            row("showcase/pages/lists/72x20/nocolor", "reapproved"),
            row("showcase/flows/selectable/x/72x20/nocolor", "reapproved"),
            row("showcase/flows/help/overlay/72x20/nocolor", "legacy"),
            row("tablepro/connections/default/72x20/nocolor", "legacy"),
            row("holla/parity/a/72x20/nocolor", "reapproved"),
        ];
        let apps = BTreeSet::from(["showcase", "tablepro"]);
        assert_eq!(
            touched_filter(&rows, &apps).as_deref(),
            Some("test(/^showcase::flows::/) | test(/^showcase::pages::/)")
        );
        assert_eq!(touched_filter(&rows, &BTreeSet::from(["tablepro"])), None);
    }

    #[test]
    fn ratchet_flags_regressions_and_new_passes() {
        let set = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<BTreeSet<_>>();
        let selected = set(&["a", "b", "c"]);
        let listed = set(&["a", "b", "z"]);
        let passing = set(&["a", "c"]);
        let (regressed, new) = ratchet_verdict(&selected, &listed, &passing);
        // `z` is listed but not selected: not judged by this run
        assert_eq!(regressed, ["b"]);
        assert_eq!(new, ["c"]);
    }

    #[test]
    fn manifest_round_trips_through_the_flat_reader() {
        let m = ManifestOut {
            version: 1,
            pending: false,
            source_commit: "a".repeat(40),
            tuiscotti_rev: "rev".into(),
            harness_sha256: "ff".into(),
            approved_by: "PO".into(),
            date: "2026-10-16".into(),
            counts: BTreeMap::from([("holla".to_string(), 3)]),
        };
        let text = render_manifest(&m);
        assert_eq!(manifest_value(&text, "pending").as_deref(), Some("false"));
        assert_eq!(manifest_value(&text, "version").as_deref(), Some("1"));
        assert_eq!(
            manifest_value(&text, "harness_sha256").as_deref(),
            Some("ff")
        );
        assert_eq!(manifest_value(&text, "holla").as_deref(), Some("3"));
        assert_eq!(manifest_value(&text, "showcase").as_deref(), Some("0"));
        // the committed skeleton reads as pending
        let skeleton = std::fs::read_to_string(root().join(ORACLE).join(MANIFEST)).unwrap();
        assert_eq!(
            manifest_value(&skeleton, "pending").as_deref(),
            Some("true")
        );
    }

    #[test]
    fn sync_args_need_a_source_commit() {
        assert!(parse_sync_args(&[]).is_err());
        assert!(parse_sync_args(&["--from"]).is_err());
        assert!(parse_sync_args(&["--from", "abc", "--bogus"]).is_err());
        let a = parse_sync_args(&[
            "--from",
            "abc",
            "--harness-dest",
            "crates/termrock-conformance/tests/visual_baseline",
            "--allow-interim",
        ])
        .unwrap();
        assert_eq!(a.from, "abc");
        assert_eq!(
            a.harness_dest,
            "crates/termrock-conformance/tests/visual_baseline"
        );
        assert!(a.allow_interim);
        assert_eq!(a.repo, root());
    }

    #[test]
    fn runs_compare_byte_for_byte() {
        let names: Vec<String> = ["a", "b", "c"].map(String::from).to_vec();
        let first = BTreeMap::from([
            ("a".to_string(), b"x".to_vec()),
            ("b".to_string(), b"y".to_vec()),
        ]);
        let second = BTreeMap::from([
            ("a".to_string(), b"x".to_vec()),
            ("b".to_string(), b"z".to_vec()),
            ("c".to_string(), b"w".to_vec()),
        ]);
        assert_eq!(
            compare_runs(&names, &first, &second),
            RunComparison {
                identical: 1,
                differ: vec!["b".into()],
                missing: vec!["c".into()],
            }
        );
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
        // every store- or list-writing command stops before doing anything
        let ci = |r: Result<(), String>| r.is_err_and(|e| e.contains("CI is set"));
        assert!(ci(seal(None, None)));
        assert!(ci(ratchet(false, None)));
        assert!(ci(sync_oracle(&["--from", "HEAD"])));
        assert!(ci(import_args(&["/nonexistent", "--oracle"])));
        unsafe { std::env::set_var("CI", "") };
        assert!(refuse_in_ci("accept").is_ok());
        unsafe { std::env::remove_var("CI") };
        assert!(refuse_in_ci("accept").is_ok());
    }
}
