use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::support;

pub const DIR: &str = "baselines/oracle";
pub const MANIFEST: &str = "ORACLE.toml";

pub const PROGRESS_DIR: &str = "progress";
pub const APPS: [&str; 4] = ["showcase", "tablepro", "jackin", "holla"];

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Manifest {
    pub version: u64,
    pub pending: bool,
    pub source_commit: String,
    pub tuiscotti_rev: String,
    pub harness_sha256: String,
    pub approved_by: String,
    pub date: String,
    pub scenarios: BTreeMap<String, u64>,
}

fn unquote(value: &str) -> Result<String, String> {
    value
        .strip_prefix('"')
        .and_then(|v| v.strip_suffix('"'))
        .filter(|v| !v.contains('"') && !v.contains('\\'))
        .map(str::to_string)
        .ok_or_else(|| format!("expected a plain \"string\", got `{value}`"))
}

impl Manifest {
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut m = Manifest::default();
        let mut seen = BTreeSet::new();
        let mut section = "";
        for (n, raw) in text.lines().enumerate() {
            let line = raw.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let at = |e: String| format!("{MANIFEST}:{}: {e}", n + 1);
            if line == "[scenarios]" {
                section = "scenarios";
                continue;
            }
            if line.starts_with('[') {
                return Err(at(format!("unknown table `{line}`")));
            }
            let (key, value) = line
                .split_once('=')
                .map(|(k, v)| (k.trim(), v.trim()))
                .ok_or_else(|| at(format!("expected `key = value`, got `{line}`")))?;
            if !seen.insert(format!("{section}.{key}")) {
                return Err(at(format!("duplicate key `{key}`")));
            }
            if section == "scenarios" {
                if !APPS.contains(&key) {
                    return Err(at(format!("unknown app `{key}`")));
                }
                let count = value
                    .parse()
                    .map_err(|_| at(format!("`{key}` must be an integer")))?;
                m.scenarios.insert(key.to_string(), count);
                continue;
            }
            match key {
                "version" => {
                    m.version = value
                        .parse()
                        .map_err(|_| at("`version` must be an integer".into()))?;
                }
                "pending" => {
                    m.pending = match value {
                        "true" => true,
                        "false" => false,
                        _ => return Err(at("`pending` must be true or false".into())),
                    };
                }
                "source_commit" => m.source_commit = unquote(value).map_err(at)?,
                "tuiscotti_rev" => m.tuiscotti_rev = unquote(value).map_err(at)?,
                "harness_sha256" => m.harness_sha256 = unquote(value).map_err(at)?,
                "approved_by" => m.approved_by = unquote(value).map_err(at)?,
                "date" => m.date = unquote(value).map_err(at)?,
                other => return Err(at(format!("unknown key `{other}`"))),
            }
        }
        for key in [
            "version",
            "pending",
            "source_commit",
            "tuiscotti_rev",
            "harness_sha256",
            "approved_by",
            "date",
        ] {
            if !seen.contains(&format!(".{key}")) {
                return Err(format!("{MANIFEST}: missing `{key}`"));
            }
        }
        for app in APPS {
            if !m.scenarios.contains_key(app) {
                return Err(format!("{MANIFEST}: [scenarios] is missing `{app}`"));
            }
        }
        Ok(m)
    }
}

pub fn root() -> PathBuf {
    support::manifest_path(DIR)
}

pub fn load() -> Result<Option<Manifest>, String> {
    let path = root().join(MANIFEST);
    match std::fs::read_to_string(&path) {
        Ok(text) => Manifest::parse(&text).map(Some),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("read {}: {e}", path.display())),
    }
}

pub fn active() -> bool {
    matches!(load(), Ok(Some(m)) if !m.pending)
}

fn walk(root: &Path, dir: &Path, skip_top: Option<&str>, out: &mut Vec<(String, PathBuf)>) {
    let entries = std::fs::read_dir(dir).unwrap_or_else(|e| panic!("list {}: {e}", dir.display()));
    for entry in entries {
        let path = entry
            .unwrap_or_else(|e| panic!("list {}: {e}", dir.display()))
            .path();
        if support::is_macos_platform_metadata(&path) {
            continue;
        }
        let rel = path
            .strip_prefix(root)
            .expect("under root")
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join("/");
        if path.is_dir() {
            if skip_top.is_some_and(|s| s == rel) {
                continue;
            }
            walk(root, &path, skip_top, out);
        } else {
            out.push((rel, path));
        }
    }
}

pub fn harness_sha256(dir: &Path) -> String {
    let mut files = Vec::new();
    walk(dir, dir, Some(PROGRESS_DIR), &mut files);
    files.sort();
    let mut hasher = Sha256::new();
    for (rel, path) in files {
        let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        hasher.update(rel.as_bytes());
        hasher.update([0]);
        hasher.update((bytes.len() as u64).to_le_bytes());
        hasher.update(&bytes);
    }
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

pub fn harness_dir() -> PathBuf {
    support::manifest_path("tests/visual_baseline")
}

pub fn pinned_tuiscotti_rev() -> String {
    let toml = std::fs::read_to_string(support::manifest_path("Cargo.toml")).expect("Cargo.toml");
    toml.lines()
        .find(|l| l.trim_start().starts_with("tuiscotti ="))
        .and_then(|l| l.split("rev = \"").nth(1))
        .and_then(|r| r.split('"').next())
        .map(str::to_string)
        .expect("Cargo.toml pins tuiscotti with `rev = \"…\"`")
}

pub struct Expect<'a> {
    pub harness: &'a Path,
    pub tuiscotti_rev: &'a str,

    pub suite: &'a BTreeSet<String>,

    pub required: &'a BTreeSet<String>,
}

pub fn check_store(store: &Path, expect: &Expect<'_>) -> Result<String, String> {
    let text = std::fs::read_to_string(store.join(MANIFEST))
        .map_err(|e| format!("{}/{MANIFEST}: {e}", store.display()))?;
    let manifest = Manifest::parse(&text)?;
    let mut files = Vec::new();
    walk(store, store, None, &mut files);
    let mut stored = BTreeSet::new();
    let mut extra = Vec::new();
    for (rel, _) in &files {
        match rel.strip_suffix(".ansi") {
            Some(name) => {
                stored.insert(name.to_string());
            }
            None if rel == MANIFEST => {}
            None => extra.push(rel.clone()),
        }
    }
    if !extra.is_empty() {
        return Err(format!(
            "the oracle holds files other than `.ansi` and {MANIFEST}: {extra:?}"
        ));
    }
    if manifest.pending {
        if !stored.is_empty() {
            return Err(format!(
                "the oracle is pending (no capture imported) but holds {} `.ansi` file(s)",
                stored.len()
            ));
        }
        if manifest.scenarios.values().any(|&n| n != 0) {
            return Err("a pending oracle counts no scenarios".into());
        }
        return Ok(format!(
            "oracle pending (v{}); the gate reads snapshots/",
            manifest.version
        ));
    }

    if manifest.tuiscotti_rev != expect.tuiscotti_rev {
        return Err(format!(
            "{MANIFEST} tuiscotti_rev {} differs from the Cargo.toml pin {}: re-capture and re-approve after a bump",
            manifest.tuiscotti_rev, expect.tuiscotti_rev
        ));
    }
    let hash = harness_sha256(expect.harness);
    if manifest.harness_sha256 != hash {
        return Err(format!(
            "tests/visual_baseline/ hashes to {hash}, the oracle was sealed for {}: review the harness change, then `cargo xtask visual seal`",
            manifest.harness_sha256
        ));
    }
    if manifest.source_commit.len() != 40
        || !manifest
            .source_commit
            .chars()
            .all(|c| c.is_ascii_hexdigit())
    {
        return Err(format!(
            "{MANIFEST} source_commit must be a full commit SHA"
        ));
    }
    if manifest.approved_by.is_empty() || manifest.date.is_empty() {
        return Err(format!("{MANIFEST} names who approved it and when"));
    }
    let orphans: Vec<_> = stored.difference(expect.suite).collect();
    if !orphans.is_empty() {
        return Err(format!(
            "oracle names not in the scenario tables: {orphans:?}"
        ));
    }
    let missing: Vec<_> = expect.required.difference(&stored).collect();
    if !missing.is_empty() {
        return Err(format!(
            "the oracle is missing {} approved scenario(s): {missing:?}",
            missing.len()
        ));
    }
    let mut counts: BTreeMap<String, u64> = APPS.iter().map(|a| (a.to_string(), 0)).collect();
    let provenance =
        tuiscotti::render::frame_from_screen(&tuiscotti::Screen::blank(1, 1), "oracle").provenance;
    for name in &stored {
        let text = std::fs::read_to_string(store.join(format!("{name}.ansi")))
            .map_err(|e| format!("read {name}.ansi: {e}"))?;
        let canonical = tuiscotti::formats::ansi::frame_from_ansi(&text, provenance.clone())
            .is_ok_and(|frame| tuiscotti::render::ansi_dump_with_cursor(&frame) == text);
        if !canonical {
            return Err(format!(
                "{name}.ansi is not a normalized SGR dump with one cursor trailer"
            ));
        }
        let app = name.split('/').next().unwrap_or_default();
        *counts.entry(app.to_string()).or_default() += 1;
    }
    if counts != manifest.scenarios {
        return Err(format!(
            "{MANIFEST} [scenarios] {:?} differ from the store {counts:?}",
            manifest.scenarios
        ));
    }
    Ok(format!(
        "oracle v{} at {}: {} scenario(s)",
        manifest.version,
        manifest.source_commit,
        stored.len()
    ))
}

pub fn integrity() {
    let all = crate::scenarios::all();
    let suite: BTreeSet<String> = all.iter().map(|s| s.name.clone()).collect();
    let required: BTreeSet<String> = all
        .iter()
        .filter(|s| !s.pending)
        .map(|s| s.name.clone())
        .collect();
    let rev = pinned_tuiscotti_rev();
    let expect = Expect {
        harness: &harness_dir(),
        tuiscotti_rev: &rev,
        suite: &suite,
        required: &required,
    };
    match check_store(&root(), &expect) {
        Ok(summary) => eprintln!("{summary}"),
        Err(e) => panic!("{DIR}: {e}"),
    }
}

pub fn store_check_accepts_and_rejects() {
    let base = support::manifest_path("target/tuiscotti/tmp/oracle-store");
    let _ = std::fs::remove_dir_all(&base);
    let harness = base.join("harness");
    std::fs::create_dir_all(&harness).unwrap();
    std::fs::write(harness.join("main.rs"), "fn main() {}").unwrap();
    let hash = harness_sha256(&harness);
    let suite: BTreeSet<String> = ["showcase/p/a/72x20/nocolor", "holla/p/b/72x20/nocolor"]
        .map(String::from)
        .into();
    let required = suite.clone();
    let expect = Expect {
        harness: &harness,
        tuiscotti_rev: "rev1",
        suite: &suite,
        required: &required,
    };
    let manifest = |pending: bool, showcase: u64, holla: u64, rev: &str, hash: &str| {
        format!(
            "version = 0\npending = {pending}\nsource_commit = \"{}\"\ntuiscotti_rev = \"{rev}\"\n\
             harness_sha256 = \"{hash}\"\napproved_by = \"t\"\ndate = \"2026-10-16\"\n\n\
             [scenarios]\nshowcase = {showcase}\ntablepro = 0\njackin = 0\nholla = {holla}\n",
            "a".repeat(40)
        )
    };
    let store = base.join("store");
    let put = |rel: &str, body: &str| {
        let path = store.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, body).unwrap();
    };

    let blank = tuiscotti::render::frame_from_screen(&tuiscotti::Screen::blank(1, 1), "t");
    let ansi = &tuiscotti::render::ansi_dump_with_cursor(&blank);

    put(MANIFEST, &manifest(true, 0, 0, "", ""));
    assert!(check_store(&store, &expect).is_ok(), "pending skeleton");
    put("showcase/p/a/72x20/nocolor.ansi", ansi);
    assert!(
        check_store(&store, &expect).is_err(),
        "pending with a capture"
    );

    put("holla/p/b/72x20/nocolor.ansi", ansi);
    put(MANIFEST, &manifest(false, 1, 1, "rev1", &hash));
    assert_eq!(
        check_store(&store, &expect),
        Ok(format!("oracle v0 at {}: 2 scenario(s)", "a".repeat(40)))
    );
    put("holla/p/b/72x20/nocolor.ansi", "x\n");
    assert!(
        check_store(&store, &expect).is_err(),
        "no cursor trailer: not canonical"
    );
    put("holla/p/b/72x20/nocolor.ansi", ansi);
    put(MANIFEST, &manifest(false, 2, 1, "rev1", &hash));
    assert!(check_store(&store, &expect).is_err(), "wrong count");
    put(MANIFEST, &manifest(false, 1, 1, "rev2", &hash));
    assert!(check_store(&store, &expect).is_err(), "tuiscotti bumped");
    put(MANIFEST, &manifest(false, 1, 1, "rev1", "00"));
    assert!(check_store(&store, &expect).is_err(), "harness changed");
    put(MANIFEST, &manifest(false, 1, 1, "rev1", &hash));
    put("showcase/p/a/72x20/nocolor.png", "x");
    assert!(check_store(&store, &expect).is_err(), "a stray file");
    std::fs::remove_file(store.join("showcase/p/a/72x20/nocolor.png")).unwrap();
    std::fs::remove_file(store.join("holla/p/b/72x20/nocolor.ansi")).unwrap();
    put(MANIFEST, &manifest(false, 1, 0, "rev1", &hash));
    assert!(check_store(&store, &expect).is_err(), "a missing approval");
}

pub fn manifest_and_hash_are_stable() {
    let text = "\
# comment
version = 1
pending = false
source_commit = \"0123456789abcdef0123456789abcdef01234567\"
tuiscotti_rev = \"abc\"
harness_sha256 = \"ff\"
approved_by = \"PO\"
date = \"2026-10-16\"

[scenarios]
showcase = 2
tablepro = 0
jackin = 0
holla = 1
";
    let m = Manifest::parse(text).expect("parses");
    assert_eq!(m.version, 1);
    assert!(!m.pending);
    assert_eq!(m.scenarios["showcase"], 2);
    assert!(Manifest::parse(&text.replace("holla = 1\n", "")).is_err());
    assert!(Manifest::parse(&format!("{text}extra = 1\n")).is_err());
    assert!(Manifest::parse(&text.replace("version = 1", "version = 1\nversion = 2")).is_err());

    let dir = support::manifest_path("target/tuiscotti/tmp/harness-hash-vector");
    let _ = std::fs::remove_dir_all(&dir);
    for (rel, body) in [
        ("a.rs", "x"),
        ("sub/b.rs", "yz"),
        ("progress/main-passing.txt", "ignored"),
    ] {
        let path = dir.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, body).unwrap();
    }
    let with_progress = harness_sha256(&dir);
    std::fs::write(dir.join("progress/main-passing.txt"), "changed").unwrap();
    assert_eq!(
        harness_sha256(&dir),
        with_progress,
        "progress/ is not hashed"
    );
    assert_eq!(
        with_progress, HASH_VECTOR,
        "the harness hash layout changed"
    );
    std::fs::write(dir.join("sub/b.rs"), "yy").unwrap();
    assert_ne!(harness_sha256(&dir), with_progress);
}

const HASH_VECTOR: &str = "58101eafd1290d1707f14178bf42b1bc8284d0a6652f423380db1c667cc381a2";
