#![deny(unsafe_op_in_unsafe_fn)]

#[cfg(not(target_os = "macos"))]
compile_error!("this helper is for the reviewed Darwin clonefile(2) implementation only");

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::CString;
use std::fs;
use std::io;
use std::os::raw::c_char;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::process;

const SOURCE_PACK_DIR: &str = "/private/tmp/termrock-reference-dco-selective-repair-root-20261010/objects.git/objects/pack";
const DEST_PACK_DIR: &str = "/private/tmp/termrock-reference-dco-bitmap-probe-observer-r10-20261010/scratch/probe.git/objects/pack";
const ALLOWLIST_TSV: &str = "/private/tmp/termrock-reference-dco-bitmap-probe-plan-observer-r10-20261010/clone-allowlist.tsv";
const EXCLUDED_NAME: &str = "tmp_pack_l6MVvt";
const EXPECTED_FILES: usize = 5052;
const EXPECTED_GROUPS: usize = 1263;
const CLONE_FLAGS: u32 = 0x0008 | 0x0004; // CLONE_NOFOLLOW_ANY | CLONE_ACL, from Apple xnu bsd/sys/clonefile.h.

#[link(name = "System")]
unsafe extern "C" {
    fn clonefile(src: *const c_char, dst: *const c_char, flags: u32) -> i32;
}

#[derive(Clone, Debug)]
struct Expected {
    name: String,
    dev: u64,
    ino: u64,
    mode: u32,
    size: u64,
    nlink: u64,
    ctime_ns: i128,
}

fn fail(message: impl std::fmt::Display) -> ! {
    eprintln!("clone-source-packs: {message}");
    process::exit(2)
}

fn parse_num<T: std::str::FromStr>(s: &str, field: &str, line: usize) -> T
where
    T::Err: std::fmt::Display,
{
    s.parse().unwrap_or_else(|e| fail(format!("invalid {field} on allowlist line {line}: {e}")))
}

fn is_pack_name(name: &str) -> Option<(&str, &str)> {
    let tail = name.strip_prefix("pack-")?;
    let (oid, ext) = tail.split_once('.')?;
    if oid.len() != 40 || !oid.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()) {
        return None;
    }
    if !matches!(ext, "pack" | "idx" | "rev" | "promisor") {
        return None;
    }
    Some((oid, ext))
}

fn metadata(path: &Path) -> fs::Metadata {
    fs::symlink_metadata(path).unwrap_or_else(|e| fail(format!("lstat {}: {e}", path.display())))
}

fn assert_directory(path: &Path, dev: Option<u64>) -> fs::Metadata {
    let m = metadata(path);
    if m.file_type().is_symlink() || !m.is_dir() {
        fail(format!("expected non-symlink directory: {}", path.display()));
    }
    if let Some(want) = dev {
        if m.dev() != want {
            fail(format!("directory is on an unexpected filesystem: {}", path.display()));
        }
    }
    m
}

fn ctime_ns(m: &fs::Metadata) -> i128 {
    i128::from(m.ctime()) * 1_000_000_000 + i128::from(m.ctime_nsec())
}

fn check_expected(path: &Path, e: &Expected) -> fs::Metadata {
    let m = metadata(path);
    let mode = m.mode();
    if m.file_type().is_symlink() || !m.is_file()
        || m.dev() != e.dev
        || m.ino() != e.ino
        || mode != e.mode
        || m.size() != e.size
        || m.nlink() != e.nlink
        || ctime_ns(&m) != e.ctime_ns
        || mode & 0o6000 != 0
    {
        fail(format!("source metadata no longer matches reviewed inventory: {}", e.name));
    }
    m
}

fn read_allowlist() -> Vec<Expected> {
    let bytes = fs::read(ALLOWLIST_TSV).unwrap_or_else(|e| fail(format!("read allowlist: {e}")));
    if bytes.len() > 2_000_000 || !bytes.ends_with(b"\n") {
        fail("allowlist size/newline guard failed");
    }
    let text = std::str::from_utf8(&bytes).unwrap_or_else(|_| fail("allowlist is not UTF-8"));
    let mut rows = Vec::with_capacity(EXPECTED_FILES);
    let mut previous: Option<String> = None;
    let mut groups: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (idx, line) in text.lines().enumerate() {
        let line_no = idx + 1;
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() != 8 {
            fail(format!("allowlist field count on line {line_no}"));
        }
        let name = f[0].to_owned();
        if previous.as_ref().is_some_and(|p| p >= &name) {
            fail(format!("allowlist is not strictly sorted at line {line_no}"));
        }
        let (oid, ext) = is_pack_name(&name).unwrap_or_else(|| fail(format!("invalid pack name on line {line_no}")));
        if f[7].len() != 64 || !f[7].bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()) {
            fail(format!("invalid SHA-256 on line {line_no}"));
        }
        let extensions = groups.entry(oid.to_owned()).or_default();
        if !extensions.insert(ext.to_owned()) {
            fail(format!("duplicate pack sidecar on line {line_no}"));
        }
        let mode = u32::from_str_radix(f[3], 8).unwrap_or_else(|e| fail(format!("invalid mode on line {line_no}: {e}")));
        rows.push(Expected {
            name: name.clone(),
            dev: parse_num(f[1], "device", line_no),
            ino: parse_num(f[2], "inode", line_no),
            mode,
            size: parse_num(f[4], "size", line_no),
            nlink: parse_num(f[5], "link count", line_no),
            ctime_ns: parse_num(f[6], "ctime", line_no),
        });
        previous = Some(name);
    }
    if rows.len() != EXPECTED_FILES || groups.len() != EXPECTED_GROUPS {
        fail(format!("allowlist cardinality mismatch: files={}, groups={}", rows.len(), groups.len()));
    }
    for (oid, exts) in groups {
        if exts.len() != 4 || !["pack", "idx", "rev", "promisor"].iter().all(|x| exts.contains(*x)) {
            fail(format!("incomplete sidecar group for {oid}"));
        }
    }
    rows
}

fn assert_source_directory_listing(rows: &[Expected]) {
    let source = Path::new(SOURCE_PACK_DIR);
    let dir_meta = assert_directory(source, None);
    if dir_meta.dev() != 16_777_234
        || dir_meta.ino() != 89_040_009
        || dir_meta.mode() != 0o40755
        || dir_meta.size() != 161_760
        || dir_meta.nlink() != 5_055
        || ctime_ns(&dir_meta) != 1_791_573_990_039_889_095
    {
        fail("source pack directory identity changed from frozen inventory");
    }
    let expected_names: BTreeSet<String> = rows.iter().map(|r| r.name.clone()).collect();
    let mut seen = BTreeSet::new();
    let mut excluded_seen = false;
    for entry in fs::read_dir(source).unwrap_or_else(|e| fail(format!("read source pack directory: {e}"))) {
        let entry = entry.unwrap_or_else(|e| fail(format!("read source directory entry: {e}")));
        let name = entry.file_name().into_string().unwrap_or_else(|_| fail("non-UTF-8 source entry"));
        let path = entry.path();
        if name == EXCLUDED_NAME {
            let m = metadata(&path);
            if m.file_type().is_symlink() || !m.is_file()
                || m.dev() != 16_777_234
                || m.ino() != 89_091_953
                || m.mode() != 0o100444
                || m.size() != 5_577_026_288
                || m.nlink() != 1
                || ctime_ns(&m) != 1_791_573_972_188_982_527
            {
                fail("excluded temporary pack metadata changed");
            }
            excluded_seen = true;
            continue;
        }
        if !expected_names.contains(&name) || !seen.insert(name) {
            fail("source pack directory has an unexpected or duplicate entry");
        }
    }
    if !excluded_seen || seen != expected_names {
        fail("source pack directory listing does not match the exact allowlist plus excluded temp pack");
    }
}

fn clone_one(e: &Expected) {
    let src = Path::new(SOURCE_PACK_DIR).join(&e.name);
    let dst = Path::new(DEST_PACK_DIR).join(&e.name);
    check_expected(&src, e);
    if dst.exists() || fs::symlink_metadata(&dst).is_ok() {
        fail(format!("destination already exists: {}", e.name));
    }
    let src_c = CString::new(src.as_os_str().as_bytes()).unwrap_or_else(|_| fail("NUL in source path"));
    let dst_c = CString::new(dst.as_os_str().as_bytes()).unwrap_or_else(|_| fail("NUL in destination path"));
    // SAFETY: paths are NUL-free CStrings, flags are Apple header constants, and the destination is required absent.
    let rc = unsafe { clonefile(src_c.as_ptr(), dst_c.as_ptr(), CLONE_FLAGS) };
    if rc != 0 {
        fail(format!("clonefile failed for {}: {}", e.name, io::Error::last_os_error()));
    }
    let dm = metadata(&dst);
    if dm.file_type().is_symlink() || !dm.is_file()
        || dm.dev() != e.dev
        || dm.ino() == e.ino
        || dm.mode() != e.mode
        || dm.size() != e.size
        || dm.nlink() != 1
    {
        fail(format!("cloned destination metadata mismatch: {}", e.name));
    }
}

fn main() {
    if std::env::args_os().len() != 1 {
        fail("this fixed-path helper accepts no arguments");
    }
    let rows = read_allowlist();
    assert_source_directory_listing(&rows);
    let source = Path::new(SOURCE_PACK_DIR);
    let dest = Path::new(DEST_PACK_DIR);
    if fs::canonicalize(source).unwrap_or_else(|e| fail(format!("canonicalize source: {e}"))) != source {
        fail("source path contains an unexpected symlink component");
    }
    if fs::canonicalize(dest).unwrap_or_else(|e| fail(format!("canonicalize destination: {e}"))) != dest {
        fail("destination path contains an unexpected symlink component");
    }
    let src_dir = assert_directory(source, None);
    let dst_dir = assert_directory(dest, Some(src_dir.dev()));
    if dst_dir.ino() == src_dir.ino() {
        fail("source and destination pack directories alias the same inode");
    }
    if fs::read_dir(dest).unwrap_or_else(|e| fail(format!("read destination pack directory: {e}"))).next().is_some() {
        fail("destination pack directory is not empty");
    }
    for row in &rows {
        check_expected(&source.join(&row.name), row);
    }
    for row in &rows {
        clone_one(row);
    }
    for row in &rows {
        check_expected(&source.join(&row.name), row);
    }
    assert_source_directory_listing(&rows);
    let got: BTreeSet<String> = fs::read_dir(dest)
        .unwrap_or_else(|e| fail(format!("read destination pack directory after clone: {e}")))
        .map(|e| e.unwrap_or_else(|e| fail(format!("read destination entry: {e}"))).file_name().into_string().unwrap_or_else(|_| fail("non-UTF-8 destination entry")))
        .collect();
    let want: BTreeSet<String> = rows.iter().map(|r| r.name.clone()).collect();
    if got != want {
        fail("destination pack directory differs from exact allowlist");
    }
    let bytes: u128 = rows.iter().map(|r| u128::from(r.size)).sum();
    println!("{{\"status\":\"CLONE_COMPLETE\",\"files\":{},\"pack_groups\":{},\"logical_bytes\":{},\"method\":\"Darwin_clonefile_CLONE_NOFOLLOW_ANY_CLONE_ACL\",\"fallback\":false}}", rows.len(), EXPECTED_GROUPS, bytes);
}
