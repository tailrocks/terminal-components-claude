//! Native confined builds with immutable input roots and fresh output roots.
use crate::capture_build::file_hash;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Confinement {
    pub(crate) backend: String,
    pub(crate) profile: String,
    #[serde(deserialize_with = "crate::capture_contract::unique_map")]
    pub(crate) environment: BTreeMap<String, String>,
    #[serde(deserialize_with = "crate::capture_contract::unique_map")]
    pub(crate) input_roots: BTreeMap<String, String>,
    pub(crate) platform: String,
    pub(crate) cargo: String,
}
fn text_command(program: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .env_clear()
        .envs(crate::capture_build::environment())
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "confinement prerequisite failed: {}",
            program.display()
        ));
    }
    String::from_utf8(output.stdout)
        .map(|s| s.trim().to_owned())
        .map_err(|e| e.to_string())
}
fn quoted(path: &Path) -> Result<String, String> {
    serde_json::to_string(path.to_str().ok_or("non-UTF8 confinement path")?)
        .map_err(|e| e.to_string())
}
/// Fingerprint a read root including symlink identities, never following escapes.
fn tree_hash(root: &Path) -> Result<String, String> {
    if root.is_file() {
        return crate::capture_contract::encoded_hash(&("file", file_hash(root)?));
    }
    fn walk(
        root: &Path,
        path: &Path,
        entries: &mut BTreeMap<String, String>,
    ) -> Result<(), String> {
        for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            let kind = entry.file_type().map_err(|e| e.to_string())?;
            let name = path
                .strip_prefix(root)
                .map_err(|e| e.to_string())?
                .to_str()
                .ok_or("non-UTF8 confinement input path")?
                .to_owned();
            if kind.is_dir() {
                entries.insert(format!("{name}/"), "directory".into());
                walk(root, &path, entries)?;
            } else if kind.is_file() {
                entries.insert(name, file_hash(&path)?);
            } else if kind.is_symlink() {
                let target = path.canonicalize().map_err(|e| e.to_string())?;
                if !target.starts_with(root) {
                    return Err(format!("input-root symlink escapes: {}", path.display()));
                }
                entries.insert(
                    name,
                    format!(
                        "symlink:{}",
                        fs::read_link(&path)
                            .map_err(|e| e.to_string())?
                            .to_str()
                            .ok_or("non-UTF8 confinement symlink target")?
                    ),
                );
            } else {
                return Err(format!("special confinement input: {}", path.display()));
            }
        }
        Ok(())
    }
    let mut entries = BTreeMap::new();
    walk(root, root, &mut entries)?;
    crate::capture_contract::encoded_hash(&("directory", entries))
}

// Cargo vendor takes an exclusive cache lock. Never run it against the home
// retained by an enclosing `cargo run`/`cargo test` process.
fn snapshot_registry(source: &Path, destination: &Path) -> Result<(), String> {
    fn copy(source: &Path, destination: &Path) -> Result<(), String> {
        fs::create_dir(destination).map_err(|e| e.to_string())?;
        for entry in fs::read_dir(source).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let kind = entry.file_type().map_err(|e| e.to_string())?;
            let target = destination.join(entry.file_name());
            if kind.is_dir() {
                copy(&entry.path(), &target)?;
            } else if kind.is_file() {
                fs::copy(entry.path(), target).map_err(|e| e.to_string())?;
            } else {
                return Err("registry snapshot contains a symlink or special file".into());
            }
        }
        Ok(())
    }
    let before = tree_hash(source)?;
    copy(source, destination)?;
    if tree_hash(source)? != before || tree_hash(destination)? != before {
        return Err("registry input changed while snapshotting".into());
    }
    Ok(())
}
impl Confinement {
    pub(crate) fn validate_shape(&self) -> Result<(), String> {
        if self.backend != "macos-seatbelt-v1"
            || self.profile.is_empty()
            || self.platform.is_empty()
            || !Path::new(&self.cargo).is_absolute()
            || self.input_roots.is_empty()
            || self.input_roots.iter().any(|(path, digest)| {
                !Path::new(path).is_absolute()
                    || digest.len() != 64
                    || !digest.bytes().all(|b| b.is_ascii_hexdigit())
            })
            || ["PATH", "HOME", "CARGO_HOME", "RUSTC"]
                .iter()
                .any(|key| self.environment.get(*key).is_none_or(String::is_empty))
        {
            return Err("missing or malformed confinement binding".into());
        }
        Ok(())
    }

    pub(crate) fn prepare(root: &Path, directory: &Path) -> Result<Self, String> {
        Self::configuration(root, directory, true)
    }
    fn configuration(root: &Path, directory: &Path, create: bool) -> Result<Self, String> {
        let root = root.canonicalize().map_err(|e| e.to_string())?;
        let directory = directory.canonicalize().map_err(|e| e.to_string())?;
        if !cfg!(target_os = "macos") {
            return Err("native confined capture build requires qualified platform adapter".into());
        }
        let toolchain = PathBuf::from(text_command(Path::new("rustc"), &["--print", "sysroot"])?)
            .canonicalize()
            .map_err(|e| e.to_string())?;
        let developer = PathBuf::from(text_command(Path::new("/usr/bin/xcode-select"), &["-p"])?)
            .canonicalize()
            .map_err(|e| e.to_string())?;
        let sdk = text_command(Path::new("/usr/bin/xcrun"), &["--show-sdk-path"])?;
        let clang = text_command(Path::new("/usr/bin/xcrun"), &["--find", "clang"])?;
        let sdk_path = PathBuf::from(&sdk)
            .canonicalize()
            .map_err(|e| e.to_string())?;
        let native_toolchain = PathBuf::from(&clang)
            .parent()
            .and_then(Path::parent)
            .and_then(Path::parent)
            .ok_or("native toolchain root missing")?
            .to_path_buf();
        let ssl_config = PathBuf::from("/private/etc/ssl/openssl.cnf")
            .canonicalize()
            .map_err(|e| e.to_string())?;
        let cargo = toolchain.join("bin/cargo");
        let vendor = directory.join("vendor");
        let vendor_home = directory.join("vendor-home");
        let registry_cache = vendor_home.join("registry/cache");
        let registry_index = vendor_home.join("registry/index");
        if create {
            let inherited = crate::capture_build::environment();
            let original_home = inherited
                .get("CARGO_HOME")
                .map(PathBuf::from)
                .or_else(|| inherited.get("HOME").map(|h| Path::new(h).join(".cargo")))
                .ok_or("Cargo registry home missing")?;
            fs::create_dir_all(vendor_home.join("registry")).map_err(|e| e.to_string())?;
            for (source, destination) in [
                (original_home.join("registry/cache"), &registry_cache),
                (original_home.join("registry/index"), &registry_index),
            ] {
                snapshot_registry(&source, destination)?;
            }
            let cache_before = tree_hash(&registry_cache)?;
            let index_before = tree_hash(&registry_index)?;
            let output = Command::new(&cargo)
                .env_clear()
                .envs(inherited)
                .env("CARGO_HOME", &vendor_home)
                .args(["vendor", "--locked", "--offline", "--versioned-dirs"])
                .arg(&vendor)
                .current_dir(&root)
                .output()
                .map_err(|e| e.to_string())?;
            fs::write(directory.join("vendor-stderr.log"), &output.stderr)
                .map_err(|e| e.to_string())?;
            if !output.status.success() {
                return Err("offline locked vendor prerequisite failed; no build executed".into());
            }
            if tree_hash(&registry_cache)? != cache_before
                || tree_hash(&registry_index)? != index_before
            {
                return Err("offline vendoring changed its immutable registry inputs".into());
            }
        }
        if create && !vendor.exists() {
            fs::create_dir(&vendor).map_err(|e| e.to_string())?;
        }
        let home = directory.join("home");
        let temp = directory.join("tmp");
        let target = directory.join("target");
        for path in [&home, &temp, &target] {
            if create {
                fs::create_dir(path).map_err(|e| e.to_string())?;
            }
        }
        let mut environment = BTreeMap::new();
        environment.insert(
            "PATH".into(),
            format!(
                "{}:{}:/usr/bin:/bin:/usr/sbin:/sbin",
                Path::new(&clang)
                    .parent()
                    .ok_or("clang parent missing")?
                    .display(),
                toolchain.join("bin").display()
            ),
        );
        let compiler_version = text_command(&toolchain.join("bin/rustc"), &["-vV"])?;
        let host = compiler_version
            .lines()
            .find_map(|line| line.strip_prefix("host: "))
            .ok_or("compiler host missing")?;
        environment.insert(
            format!(
                "CARGO_TARGET_{}_LINKER",
                host.replace('-', "_").to_uppercase()
            ),
            clang.clone(),
        );
        for (key, value) in [
            ("HOME", home.to_string_lossy().into_owned()),
            ("CARGO_HOME", home.to_string_lossy().into_owned()),
            ("TMPDIR", temp.to_string_lossy().into_owned()),
            (
                "RUSTC",
                toolchain.join("bin/rustc").to_string_lossy().into_owned(),
            ),
            ("DEVELOPER_DIR", developer.to_string_lossy().into_owned()),
            ("SDKROOT", sdk),
            ("LC_ALL", "C".into()),
            ("CARGO_TERM_COLOR", "never".into()),
            ("CARGO_INCREMENTAL", "0".into()),
        ] {
            environment.insert(key.into(), value);
        }
        let mut input_roots = BTreeMap::new();
        for path in [
            &vendor,
            &toolchain,
            &native_toolchain,
            &sdk_path,
            &ssl_config,
            &registry_cache,
            &registry_index,
        ] {
            input_roots.insert(path.to_string_lossy().into_owned(), tree_hash(path)?);
        }
        let read_roots: Vec<PathBuf> = [
            "/System",
            "/usr/bin",
            "/usr/lib",
            "/usr/sbin",
            "/bin",
            "/sbin",
            "/Library/Apple",
            "/private/var/db/dyld",
            "/dev/fd",
            "/dev/null",
            "/dev/zero",
            "/dev/urandom",
        ]
        .into_iter()
        .map(PathBuf::from)
        .chain([
            root.to_path_buf(),
            vendor,
            toolchain,
            native_toolchain,
            sdk_path,
            ssl_config,
            home.clone(),
            temp.clone(),
            target.clone(),
        ])
        .collect();
        let mut exceptions = Vec::new();
        let mut ancestors = std::collections::BTreeSet::new();
        for path in &read_roots {
            exceptions.push(format!("(subpath {})", quoted(path)?));
            for ancestor in path.ancestors().skip(1) {
                ancestors.insert(ancestor.to_path_buf());
            }
        }
        for ancestor in root.ancestors() {
            for name in ["config", "config.toml"] {
                let config = ancestor.join(".cargo").join(name);
                exceptions.push(format!("(literal {})", quoted(&config)?));
                ancestors.insert(ancestor.join(".cargo"));
            }
        }
        for path in ancestors {
            exceptions.push(format!("(literal {})", quoted(&path)?));
        }
        let writes = [home, temp, target]
            .iter()
            .map(|p| Ok(format!("(subpath {})", quoted(p)?)))
            .collect::<Result<Vec<_>, String>>()?
            .join(" ");
        let profile = format!(
            "(version 1) (deny default) (allow process*) (allow sysctl-read) (deny network*) (allow file-read* {}) (deny file-read* (subpath {}) (subpath {})) (allow file-write* {} (literal \"/dev/null\"))",
            exceptions.join(" "),
            quoted(&root.join(".git"))?,
            quoted(&root.join("target"))?,
            writes
        );
        Ok(Self {
            backend: "macos-seatbelt-v1".into(),
            profile,
            environment,
            input_roots,
            platform: text_command(Path::new("/usr/bin/sw_vers"), &[])?,
            cargo: cargo.to_string_lossy().into_owned(),
        })
    }
    pub(crate) fn command(&self, root: &Path, argv: &[String]) -> Command {
        let mut command = Command::new("/usr/bin/sandbox-exec");
        command
            .args(["-p", &self.profile])
            .arg(&self.cargo)
            .args(&argv[1..])
            .current_dir(root)
            .env_clear()
            .envs(&self.environment);
        command
    }
    pub(crate) fn verify_inputs(&self, root: &Path, directory: &Path) -> Result<(), String> {
        let expected = Self::configuration(root, directory, false)?;
        if *self != expected {
            return Err("confined input roots, policy, or environment changed".into());
        }
        Ok(())
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::ffi::OsStringExt;

    #[test]
    fn non_utf8_policy_paths_fail_instead_of_colliding() {
        let path = PathBuf::from(std::ffi::OsString::from_vec(vec![0xff]));
        assert!(quoted(&path).unwrap_err().contains("non-UTF8"));
    }
}
