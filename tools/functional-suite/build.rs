use std::env;
use std::path::PathBuf;
use std::process::Command;

const EXPORT_LOADER: &str = r###"
import hashlib
import json
import os
import pathlib
import re
import stat
import sys

def fail(message):
    raise SystemExit(message)

def unique_pairs(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            fail("duplicate manifest key")
        result[key] = value
    return result

def safe_read(path):
    if not path.is_absolute() or ".." in path.parts:
        fail("unsafe export file path")
    components = str(path).split("/")
    if components[0] != "" or any(part in ("", ".", "..") for part in components[1:]):
        fail("unsafe export file path")
    descriptor = os.open("/", os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    try:
        for component in path.parts[1:-1]:
            info = os.stat(component, dir_fd=descriptor, follow_symlinks=False)
            if stat.S_ISLNK(info.st_mode) or not stat.S_ISDIR(info.st_mode):
                fail("symlink or non-directory in export file path")
            next_descriptor = os.open(component, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=descriptor)
            os.close(descriptor)
            descriptor = next_descriptor
        info = os.stat(path.parts[-1], dir_fd=descriptor, follow_symlinks=False)
        if stat.S_ISLNK(info.st_mode) or not stat.S_ISREG(info.st_mode):
            fail("export file is not regular")
        file_descriptor = os.open(path.parts[-1], os.O_RDONLY | os.O_NOFOLLOW, dir_fd=descriptor)
        try:
            if not stat.S_ISREG(os.fstat(file_descriptor).st_mode):
                fail("export file is not regular")
            chunks = []
            while True:
                chunk = os.read(file_descriptor, 1024 * 1024)
                if not chunk:
                    return b"".join(chunks)
                chunks.append(chunk)
        finally:
            os.close(file_descriptor)
    finally:
        os.close(descriptor)

arguments = sys.argv[1:]
if len(arguments) < 4:
    fail("missing export authentication arguments")
manifest_path = pathlib.Path(arguments[0])
helper_path = pathlib.Path(arguments[1])
expected = arguments[2]
helper_arguments = arguments[3:]
if re.fullmatch(r"[0-9a-f]{64}", expected) is None:
    fail("invalid export seal")
if manifest_path.parent != helper_path.parent or manifest_path.name != "export-manifest.json" or helper_path.name != "export.py":
    fail("unexpected export helper paths")
manifest_bytes = safe_read(manifest_path)
if hashlib.sha256(manifest_bytes).hexdigest() != expected:
    fail("export manifest seal mismatch")
try:
    manifest = json.loads(manifest_bytes.decode("utf-8"), object_pairs_hook=unique_pairs)
except (UnicodeDecodeError, json.JSONDecodeError) as error:
    fail("invalid sealed export manifest: " + str(error))
files = manifest.get("files") if isinstance(manifest, dict) else None
if not isinstance(files, list):
    fail("sealed export manifest has no file entries")
helpers = [entry for entry in files if isinstance(entry, dict) and entry.get("path") == "export.py"]
if len(helpers) != 1:
    fail("sealed export manifest must name one helper")
entry = helpers[0]
if set(entry) != {"path", "mode", "size", "sha256"}:
    fail("sealed helper entry has unknown fields")
if type(entry["size"]) is not int or entry["size"] < 0 or re.fullmatch(r"[0-9a-f]{64}", entry["sha256"]) is None:
    fail("sealed helper entry is malformed")
helper_bytes = safe_read(helper_path)
if len(helper_bytes) != entry["size"] or hashlib.sha256(helper_bytes).hexdigest() != entry["sha256"]:
    fail("sealed helper digest mismatch")
sys.argv = [str(helper_path), *helper_arguments]
exec(compile(helper_bytes, str(helper_path), "exec"), {"__name__": "__main__", "__file__": str(helper_path)})
"###;

fn main() {
    println!("cargo:rerun-if-env-changed=TERMROCK_SUITE_EXPORT_SHA256");
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest = root.join("export-manifest.json");
    let helper = root.join("export.py");
    let seal = env::var("TERMROCK_SUITE_EXPORT_SHA256")
        .unwrap_or_else(|_| panic!("TERMROCK_SUITE_EXPORT_SHA256 is required"));
    if seal.len() != 64
        || !seal
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        panic!("TERMROCK_SUITE_EXPORT_SHA256 must be lowercase hexadecimal");
    }
    let output = Command::new("python3")
        .arg("-I")
        .arg("-c")
        .arg(EXPORT_LOADER)
        .arg(&manifest)
        .arg(&helper)
        .arg(&seal)
        .arg("verify")
        .arg("--root")
        .arg(&root)
        .arg("--sha256")
        .arg(&seal)
        .arg("--cargo")
        .output()
        .unwrap_or_else(|error| panic!("cannot run isolated export verifier: {error}"));
    if !output.status.success() {
        panic!(
            "sealed export verification failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let directives = String::from_utf8(output.stdout)
        .unwrap_or_else(|error| panic!("export verifier returned invalid UTF-8: {error}"));
    for directive in directives.lines() {
        if !directive.starts_with("cargo:rerun-if-changed=") {
            panic!("export verifier returned an invalid Cargo directive");
        }
        println!("{directive}");
    }
    println!("cargo:rustc-env=TERMROCK_SUITE_EXPORT_SHA256={seal}");
}
