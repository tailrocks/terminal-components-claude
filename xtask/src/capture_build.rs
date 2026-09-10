//! Build application binaries and verify their source/compiler-artifact binding.
#[cfg(all(test, target_os = "macos"))]
use crate::capture_contract::hash;
use crate::capture_contract::{Artifact, BuildManifest, Source};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn command(root: &Path, args: &[String]) -> Command {
    let mut command = Command::new(&args[0]);
    command
        .args(&args[1..])
        .current_dir(root)
        .env_clear()
        .envs(environment());
    command
}
pub(crate) fn environment() -> BTreeMap<String, String> {
    let mut env = BTreeMap::new();
    for key in [
        "PATH",
        "HOME",
        "TMPDIR",
        "RUSTUP_HOME",
        "CARGO_HOME",
        "RUSTUP_TOOLCHAIN",
    ] {
        if let Ok(value) = std::env::var(key) {
            env.insert(key.into(), value);
        }
    }
    env.insert("LC_ALL".into(), "C".into());
    env.insert("CARGO_TERM_COLOR".into(), "never".into());
    env
}
fn env_hash(root: &Path) -> Result<String, String> {
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let mut env = environment();
    let home = env
        .get("CARGO_HOME")
        .map(PathBuf::from)
        .or_else(|| env.get("HOME").map(|h| Path::new(h).join(".cargo")));
    if let Some(home) = home {
        for name in ["config", "config.toml"] {
            let path = home.join(name);
            if path.exists() {
                env.insert(format!("config:{name}"), file_hash(&path)?);
            }
        }
    }
    for ancestor in root.ancestors() {
        for name in ["config", "config.toml"] {
            let path = ancestor.join(".cargo").join(name);
            let value = if path.exists() {
                file_hash(&path)?
            } else {
                "absent".into()
            };
            env.insert(format!("ancestor-config:{}", path.display()), value);
        }
    }
    crate::capture_contract::encoded_hash(&env)
}
fn output(root: &Path, args: &[&str]) -> Result<String, String> {
    let args: Vec<_> = args.iter().map(|s| (*s).into()).collect();
    let result = command(root, &args).output().map_err(|e| e.to_string())?;
    if !result.status.success() {
        return Err(format!("{} failed ({})", args.join(" "), result.status));
    }
    String::from_utf8(result.stdout).map_err(|e| e.to_string())
}
pub(crate) fn file_hash(path: &Path) -> Result<String, String> {
    let mut file = crate::open_capture_artifact(path)?;
    use sha2::Digest;
    use std::io::Read;
    let mut digest = sha2::Sha256::new();
    let mut bytes = [0u8; 65536];
    loop {
        let count = file.read(&mut bytes).map_err(|e| e.to_string())?;
        if count == 0 {
            break;
        }
        digest.update(&bytes[..count]);
    }
    Ok(format!("{:x}", digest.finalize()))
}
pub(crate) fn source(root: &Path) -> Result<Source, String> {
    let revision = output(root, &["git", "rev-parse", "HEAD"])?
        .trim()
        .to_owned();
    let mut entries = BTreeMap::new();
    source_files(root, root, &mut entries)?;
    if entries.is_empty() {
        return Err("empty source fingerprint".into());
    }
    Ok(Source {
        revision,
        fingerprint: crate::capture_contract::encoded_hash(&entries)?,
        lock_sha256: file_hash(&root.join("Cargo.lock"))?,
    })
}
// Git ignore rules are not compiler input boundaries. Hash ignored files too.
fn source_files(
    root: &Path,
    directory: &Path,
    entries: &mut BTreeMap<String, String>,
) -> Result<(), String> {
    for entry in fs::read_dir(directory).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let relative = path.strip_prefix(root).map_err(|e| e.to_string())?;
        if relative == Path::new(".git") || relative.starts_with("target") {
            continue;
        }
        let name = relative.to_str().ok_or("non-UTF8 source input path")?;
        let kind = entry.file_type().map_err(|e| e.to_string())?;
        if kind.is_dir() {
            entries.insert(format!("{name}/"), "directory".into());
            source_files(root, &path, entries)?;
        } else if kind.is_file() {
            entries.insert(name.to_owned(), file_hash(&path)?);
        } else {
            return Err(format!(
                "unattested source symlink or special file: {}",
                path.display()
            ));
        }
    }
    Ok(())
}
fn validate_supported_build(root: &Path) -> Result<(), String> {
    let canonical_root = root.canonicalize().map_err(|e| e.to_string())?;
    let graph: cargo_metadata::Metadata = crate::capture_contract::strict_json(
        output(
            root,
            &["cargo", "metadata", "--locked", "--format-version", "1"],
        )?
        .as_bytes(),
    )?;
    for package in graph.packages {
        // Build scripts and procedural macros execute arbitrary host code. Neither
        // Git nor rustc dep-info attests their undeclared filesystem reads.
        if package.source.is_none()
            && package.targets.iter().any(|target| {
                target.kind.iter().any(|kind| {
                    matches!(
                        kind,
                        cargo_metadata::TargetKind::CustomBuild
                            | cargo_metadata::TargetKind::ProcMacro
                    )
                })
            })
        {
            return Err(format!(
                "unsupported build-time executable input boundary: {} (build script or procedural macro)",
                package.name
            ));
        }
        if package.source.is_none()
            && !package
                .manifest_path
                .as_std_path()
                .canonicalize()
                .map_err(|e| e.to_string())?
                .starts_with(&canonical_root)
        {
            return Err(format!(
                "external local dependency cannot be attested: {}",
                package.name
            ));
        }
    }
    Ok(())
}
// Rust/Cargo dep-info binds declared Rust inputs (including include_str!/include_bytes!).
// Arbitrary build-time executables are rejected by validate_supported_build.
// Reject syntax we cannot attest instead of silently omitting an input.
fn compiler_inputs(root: &Path, target: &Path) -> Result<BTreeMap<String, String>, String> {
    fn visit(
        root: &Path,
        directory: &Path,
        inputs: &mut BTreeMap<String, String>,
    ) -> Result<(), String> {
        for entry in fs::read_dir(directory).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            let kind = entry.file_type().map_err(|e| e.to_string())?;
            if kind.is_dir() {
                visit(root, &path, inputs)?;
            } else if path.extension().is_some_and(|ext| ext == "d") {
                let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
                let line = text.lines().next().ok_or("empty compiler dep-info")?;
                let (_, dependencies) = line
                    .split_once(": ")
                    .ok_or("unsupported compiler dep-info")?;
                let mut words = Vec::new();
                let mut word = String::new();
                let mut escaped = false;
                for ch in dependencies.chars() {
                    if escaped {
                        word.push(ch);
                        escaped = false;
                    } else if ch == '\\' {
                        escaped = true;
                    } else if ch.is_whitespace() {
                        if !word.is_empty() {
                            words.push(std::mem::take(&mut word));
                        }
                    } else {
                        word.push(ch);
                    }
                }
                if escaped {
                    return Err("unsupported continued compiler dep-info".into());
                }
                if !word.is_empty() {
                    words.push(word);
                }
                for word in words {
                    let input = root.join(word).canonicalize().map_err(|e| e.to_string())?;
                    inputs.insert(input.to_string_lossy().into_owned(), file_hash(&input)?);
                }
            }
        }
        Ok(())
    }
    let mut inputs = BTreeMap::new();
    visit(root, target, &mut inputs)?;
    if inputs.is_empty() {
        return Err("missing compiler input evidence".into());
    }
    Ok(inputs)
}
fn metadata(root: &Path) -> Result<cargo_metadata::Metadata, String> {
    crate::capture_contract::strict_json(
        output(
            root,
            &[
                "cargo",
                "metadata",
                "--locked",
                "--no-deps",
                "--format-version",
                "1",
            ],
        )?
        .as_bytes(),
    )
}
fn build_args(target: &Path, host: &str) -> Vec<String> {
    let mut args = vec![
        "cargo".into(),
        "build".into(),
        "--locked".into(),
        "--message-format=json".into(),
        "--profile".into(),
        "dev".into(),
        "--target".into(),
        host.into(),
        "--target-dir".into(),
        target.to_string_lossy().into_owned(),
    ];
    args.extend([
        "--offline".into(),
        "--config".into(),
        "source.crates-io.replace-with=\"capture-vendor\"".into(),
        "--config".into(),
        format!(
            "source.capture-vendor.directory={:?}",
            target
                .parent()
                .expect("target parent")
                .join("vendor")
                .to_string_lossy()
        ),
    ]);
    for app in &crate::app_inventory::get()
        .expect("validated inventory")
        .apps
    {
        args.extend([
            "-p".into(),
            app.package.into(),
            "--bin".into(),
            app.bin.into(),
        ]);
    }
    args
}
fn artifacts(messages: &str, metadata: &cargo_metadata::Metadata) -> Result<Vec<Artifact>, String> {
    // Preserve Cargo's plain-text diagnostics, but never let recognized JSON
    // records silently discard duplicate fields before Message parsing.
    for line in messages.lines() {
        if serde_json::from_str::<serde_json::Value>(line).is_ok() {
            let _: serde_json::Value = crate::capture_contract::strict_json(line.as_bytes())?;
        }
    }
    let inventory = crate::app_inventory::get()?;
    let mut artifacts = Vec::new();
    let mut finished = 0;
    for message in cargo_metadata::Message::parse_stream(messages.as_bytes()) {
        match message.map_err(|e| e.to_string())? {
            cargo_metadata::Message::BuildFinished(done) => {
                if !done.success {
                    return Err("Cargo build failed".into());
                }
                finished += 1;
            }
            cargo_metadata::Message::CompilerArtifact(artifact) => {
                if artifact.profile.test
                    || !artifact
                        .target
                        .kind
                        .contains(&cargo_metadata::TargetKind::Bin)
                {
                    continue;
                }
                let Some(app) = inventory
                    .apps
                    .iter()
                    .find(|a| a.bin == artifact.target.name)
                else {
                    continue;
                };
                let package = metadata
                    .workspace_packages()
                    .into_iter()
                    .find(|p| p.name.as_str() == app.package)
                    .ok_or("build package absent")?;
                if package.id != artifact.package_id {
                    return Err("compiler artifact has wrong package owner".into());
                }
                let target = package
                    .targets
                    .iter()
                    .find(|t| {
                        t.name == app.bin && t.kind.contains(&cargo_metadata::TargetKind::Bin)
                    })
                    .ok_or("required bin target absent")?;
                if !target
                    .required_features
                    .iter()
                    .all(|f| artifact.features.contains(f))
                {
                    return Err("binary required-features inactive in actual build".into());
                }
                let executable = artifact
                    .executable
                    .ok_or("bin compiler artifact has no executable")?
                    .into_std_path_buf();
                artifacts.push(Artifact {
                    app: app.id.into(),
                    package_id: artifact.package_id.to_string(),
                    target: artifact.target.name,
                    sha256: file_hash(&executable)?,
                    executable: executable.to_string_lossy().into_owned(),
                    features: artifact.features,
                    profile: serde_json::to_value(artifact.profile).map_err(|e| e.to_string())?,
                });
            }
            cargo_metadata::Message::TextLine(line) if !line.trim().is_empty() => {
                return Err("unexpected non-JSON Cargo build output".into());
            }
            _ => {}
        }
    }
    if finished != 1 {
        return Err("missing or duplicate successful Cargo build-finished record".into());
    }
    artifacts.sort_by(|a, b| a.app.cmp(&b.app));
    Ok(artifacts)
}
fn required() -> Result<Vec<&'static str>, String> {
    Ok(crate::app_inventory::get()?
        .apps
        .iter()
        .map(|a| a.id)
        .collect())
}
fn write_new<T: serde::Serialize>(path: &Path, value: &T) -> Result<(), String> {
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?;
    file.write_all(&bytes).map_err(|e| e.to_string())
}
pub(crate) fn build(root: &Path, directory: &Path) -> Result<PathBuf, String> {
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    if !directory.is_absolute() || directory.starts_with(&root) {
        return Err("capture build output must be a fresh absolute external directory".into());
    }
    validate_supported_build(&root)?;
    let metadata = metadata(&root)?;
    crate::app_inventory::get()?.validate_targets(&metadata)?;
    fs::create_dir(directory).map_err(|e| e.to_string())?;
    let directory = directory.canonicalize().map_err(|e| e.to_string())?;
    if directory.starts_with(&root) {
        return Err("capture build output resolves inside source".into());
    }
    let before = source(&root)?;
    let cargo = output(&root, &["cargo", "--version"])?;
    let rustc = output(&root, &["rustc", "-vV"])?;
    let environment_sha256 = env_hash(&root)?;
    let host_target = rustc
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .ok_or("rustc host missing")?
        .to_owned();
    let confinement = crate::capture_confinement::Confinement::prepare(&root, &directory)?;
    let argv = build_args(&directory.join("target"), &host_target);
    let result = confinement
        .command(&root, &argv)
        .output()
        .map_err(|e| e.to_string())?;
    let messages = directory.join("cargo-messages.jsonl");
    let stderr = directory.join("cargo-stderr.log");
    fs::write(&messages, &result.stdout).map_err(|e| e.to_string())?;
    fs::write(&stderr, &result.stderr).map_err(|e| e.to_string())?;
    if !result.status.success() {
        return Err(format!(
            "capture Cargo build failed ({}); evidence retained",
            result.status
        ));
    }
    let parsed = artifacts(
        std::str::from_utf8(&result.stdout).map_err(|e| e.to_string())?,
        &metadata,
    )?;
    let manifest = BuildManifest {
        schema: 1,
        source: before,
        cargo,
        rustc,
        host_target,
        environment_sha256,
        compiler_inputs: compiler_inputs(&root, &directory.join("target"))?,
        confinement,
        argv,
        status: 0,
        messages_path: messages.to_string_lossy().into_owned(),
        messages_sha256: file_hash(&messages)?,
        stderr_path: stderr.to_string_lossy().into_owned(),
        stderr_sha256: file_hash(&stderr)?,
        artifacts: parsed,
    };
    manifest.validate(&required()?)?;
    if source(&root)? != manifest.source {
        return Err("source changed during build".into());
    }
    let path = directory.join("build-manifest.json");
    write_new(&path, &manifest)?;
    verify(&root, &path)?;
    Ok(path)
}
pub(crate) fn verify(root: &Path, path: &Path) -> Result<BuildManifest, String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    let manifest: BuildManifest = crate::capture_contract::strict_json(&bytes)?;
    manifest.validate(&required()?)?;
    if source(root)? != manifest.source
        || output(root, &["cargo", "--version"])? != manifest.cargo
        || output(root, &["rustc", "-vV"])? != manifest.rustc
        || env_hash(root)? != manifest.environment_sha256
    {
        return Err("stale source/lock/toolchain/environment build manifest".into());
    }
    if !manifest
        .rustc
        .lines()
        .any(|line| line == format!("host: {}", manifest.host_target))
    {
        return Err("host target differs from compiler version".into());
    }
    let directory = path
        .parent()
        .ok_or("build manifest directory missing")?
        .canonicalize()
        .map_err(|e| e.to_string())?;
    let target = directory.join("target");
    if manifest.argv != build_args(&target, &manifest.host_target)
        || Path::new(&manifest.messages_path) != directory.join("cargo-messages.jsonl")
        || Path::new(&manifest.stderr_path) != directory.join("cargo-stderr.log")
    {
        return Err("build command/transcript path differs".into());
    }
    if file_hash(Path::new(&manifest.messages_path))? != manifest.messages_sha256
        || file_hash(Path::new(&manifest.stderr_path))? != manifest.stderr_sha256
    {
        return Err("build transcript changed".into());
    }
    validate_supported_build(root)?;
    if compiler_inputs(root, &target)? != manifest.compiler_inputs {
        return Err("compiler inputs changed".into());
    }
    let metadata = metadata(root)?;
    crate::app_inventory::get()?.validate_targets(&metadata)?;
    let actual = artifacts(
        &fs::read_to_string(&manifest.messages_path).map_err(|e| e.to_string())?,
        &metadata,
    )?;
    if serde_json::to_value(&actual).map_err(|e| e.to_string())?
        != serde_json::to_value(&manifest.artifacts).map_err(|e| e.to_string())?
    {
        return Err("manifest differs from actual compiler artifacts".into());
    }
    for artifact in &manifest.artifacts {
        let executable = Path::new(&artifact.executable)
            .canonicalize()
            .map_err(|e| e.to_string())?;
        if !executable.starts_with(&target) || file_hash(&executable)? != artifact.sha256 {
            return Err("wrong or stale build executable".into());
        }
    }
    manifest.confinement.verify_inputs(root, &directory)?;
    Ok(manifest)
}
pub(crate) fn cli(args: &[String]) -> Result<(), String> {
    match args {
        [flag, path] if flag == "--output" => {
            println!("{}", build(&crate::root(), Path::new(path))?.display());
            Ok(())
        }
        [flag, path] if flag == "--verify" => {
            verify(&crate::root(), Path::new(path))?;
            println!("current source and compiler build artifacts verified; no capture claimed");
            Ok(())
        }
        _ => Err(
            "usage: capture-build --output ABSENT_EXTERNAL_DIR | --verify BUILD_MANIFEST".into(),
        ),
    }
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;
    #[test]
    fn actual_compiler_artifacts_bind_source_lock_binary_and_unique_target() {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory =
            std::env::temp_dir().join(format!("capture-build-{}-{nonce}", std::process::id()));
        let root = directory.join("source");
        fs::create_dir_all(&root).unwrap();
        let inventory = crate::app_inventory::get().unwrap();
        let members: Vec<_> = inventory.apps.iter().map(|a| a.dir).collect();
        fs::write(
            root.join("Cargo.toml"),
            format!("[workspace]\nresolver=\"2\"\nmembers={members:?}\n"),
        )
        .unwrap();
        for app in &inventory.apps {
            let dir = root.join(app.dir);
            fs::create_dir_all(dir.join("src")).unwrap();
            fs::create_dir(dir.join("tests")).unwrap();
            fs::write(dir.join("Cargo.toml"),format!("[package]\nname=\"{}\"\nversion=\"0.0.0\"\nedition=\"2021\"\n[lib]\nname=\"{}\"\n",app.package,app.lib)).unwrap();
            fs::write(dir.join("src/lib.rs"), "").unwrap();
            fs::write(
                dir.join("src/main.rs"),
                "fn main(){println!(\"real binary fixture\");}\n",
            )
            .unwrap();
            fs::write(dir.join("tests/perf.rs"), "#[test] fn fixture(){}\n").unwrap();
        }
        // Exercise real registry build scripts and a procedural macro under the
        // same confinement as production, with the workspace's cached versions.
        let holla_manifest = root.join("apps/holla/Cargo.toml");
        let holla_toml = fs::read_to_string(&holla_manifest).unwrap();
        fs::write(&holla_manifest, format!("{holla_toml}\n[dependencies]\nserde={{version=\"=1.0.229\",features=[\"derive\"]}}\nsyn=\"=3.0.4\"\nquote=\"=1.0.47\"\nproc-macro2=\"=1.0.107\"\nunicode-ident=\"=1.0.24\"\n")).unwrap();
        output(&root, &["cargo", "generate-lockfile", "--offline"]).unwrap();
        output(&root, &["git", "init", "-q"]).unwrap();
        output(&root, &["git", "add", "."]).unwrap();
        output(
            &root,
            &[
                "git",
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "-c",
                "commit.gpgsign=false",
                "commit",
                "-s",
                "-qm",
                "capture build fixture",
                "-m",
                "Co-authored-by: Codex <codex@openai.com>",
            ],
        )
        .unwrap();
        fs::write(root.join(".gitignore"), "payload.txt\n").unwrap();
        let payload = root.join("apps/holla/src/payload.txt");
        fs::write(&payload, "original compiler input").unwrap();
        fs::write(
            root.join("apps/holla/src/main.rs"),
            "#[derive(serde::Serialize)] struct Marker;\nfn main(){println!(\"{}\", include_str!(\"payload.txt\"));}\n",
        )
        .unwrap();
        fs::create_dir(root.join("shots")).unwrap();
        let excluded_input = root.join("shots/compiler-input.txt");
        fs::write(&excluded_input, "compiler observed excluded input").unwrap();
        let main_path = root.join("apps/holla/src/main.rs");
        let main = fs::read_to_string(&main_path).unwrap();
        fs::write(
            &main_path,
            format!(
                "{main}\nconst _: &str = include_str!(\"../../../shots/compiler-input.txt\");\n"
            ),
        )
        .unwrap();
        let path = build(&root, &directory.join("build")).unwrap();
        fs::write(&excluded_input, "changed excluded input").unwrap();
        assert!(verify(&root, &path).is_err());
        fs::write(&excluded_input, "compiler observed excluded input").unwrap();
        fs::write(&payload, "changed compiler input").unwrap();
        assert!(
            verify(&root, &path).is_err(),
            "changed ignored compiler input must invalidate source provenance"
        );
        fs::write(&payload, "original compiler input").unwrap();
        let config_dir = directory.join(".cargo");
        fs::create_dir(&config_dir).unwrap();
        fs::write(config_dir.join("config.toml"), "[build]\njobs=1\n").unwrap();
        assert!(
            verify(&root, &path).is_err(),
            "new applicable parent config must invalidate build"
        );
        fs::remove_dir_all(&config_dir).unwrap();
        let external = directory.join("external");
        fs::create_dir_all(external.join("src")).unwrap();
        fs::write(
            external.join("Cargo.toml"),
            "[package]\nname=\"external-input\"\nversion=\"0.0.0\"\nedition=\"2021\"\n",
        )
        .unwrap();
        fs::write(external.join("src/lib.rs"), "pub const VALUE:u8=1;\n").unwrap();
        let manifest_path = root.join("apps/holla/Cargo.toml");
        let saved_manifest = fs::read_to_string(&manifest_path).unwrap();
        let saved_lock = fs::read(root.join("Cargo.lock")).unwrap();
        fs::write(
            &manifest_path,
            format!(
                "{saved_manifest}\nexternal-input={{path={:?}}}\n",
                external.to_str().unwrap()
            ),
        )
        .unwrap();
        output(&root, &["cargo", "generate-lockfile", "--offline"]).unwrap();
        assert!(
            build(&root, &directory.join("external-build"))
                .unwrap_err()
                .contains("external local dependency")
        );
        assert!(!directory.join("external-build").exists());
        fs::write(&manifest_path, saved_manifest).unwrap();
        fs::write(root.join("Cargo.lock"), saved_lock).unwrap();

        // Preserve the undeclared excluded-input bug class: this program would
        // otherwise generate different source without declaring its dependency.
        let script = root.join("apps/holla/build.rs");
        fs::write(&script, r#"fn main() {
            let input = std::fs::read_to_string("../../shots/compiler-input.txt").unwrap();
            std::fs::write(std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("generated.rs"), format!("const VALUE: &str = {input:?};")).unwrap();
        }"#).unwrap();
        let error = build(&root, &directory.join("script-build")).unwrap_err();
        assert!(
            error.contains("unsupported build-time executable"),
            "{error}"
        );
        assert!(!directory.join("script-build").exists());
        fs::remove_file(&script).unwrap();
        let proc_manifest = root.join("apps/holla/Cargo.toml");
        let saved_proc_manifest = fs::read_to_string(&proc_manifest).unwrap();
        fs::write(
            &proc_manifest,
            saved_proc_manifest.replace("[lib]", "[lib]\nproc-macro=true"),
        )
        .unwrap();
        assert!(
            build(&root, &directory.join("macro-build"))
                .unwrap_err()
                .contains("unsupported build-time executable")
        );
        assert!(!directory.join("macro-build").exists());
        fs::write(&proc_manifest, saved_proc_manifest).unwrap();
        let manifest = verify(&root, &path).unwrap();
        assert_eq!(manifest.artifacts.len(), 4);
        let outside = directory.join("unattested.txt");
        fs::write(&outside, "outside sandbox input").unwrap();
        let denied = Command::new("/usr/bin/sandbox-exec")
            .args(["-p", &manifest.confinement.profile, "/bin/cat"])
            .arg(&outside)
            .output()
            .unwrap();
        assert!(!denied.status.success());
        let denied = Command::new("/usr/bin/sandbox-exec")
            .args(["-p", &manifest.confinement.profile, "/usr/bin/touch"])
            .arg(root.join("sandbox-source-write"))
            .output()
            .unwrap();
        assert!(!denied.status.success());
        assert!(!root.join("sandbox-source-write").exists());

        let run = Command::new(&manifest.artifacts[0].executable)
            .output()
            .unwrap();
        assert!(run.status.success());
        assert_eq!(run.stdout, b"original compiler input\n");
        let original = fs::read(&path).unwrap();
        for mutation in 0..3 {
            let mut changed = manifest.clone();
            match mutation {
                0 => changed.source.fingerprint = hash(b"wrong source"),
                1 => changed.artifacts.push(changed.artifacts[0].clone()),
                _ => changed.argv.push("--all-features".into()),
            };
            fs::write(&path, serde_json::to_vec(&changed).unwrap()).unwrap();
            assert!(verify(&root, &path).is_err());
        }
        fs::write(&path, &original).unwrap();
        let binary = Path::new(&manifest.artifacts[0].executable);
        let binary_bytes = fs::read(binary).unwrap();
        fs::write(binary, b"wrong binary").unwrap();
        assert!(verify(&root, &path).is_err());
        fs::write(binary, binary_bytes).unwrap();
        let lock = root.join("Cargo.lock");
        let lock_bytes = fs::read(&lock).unwrap();
        fs::write(&lock, b"changed lock").unwrap();
        assert!(verify(&root, &path).is_err());
        fs::write(&lock, lock_bytes).unwrap();
        let messages = Path::new(&manifest.messages_path);
        let saved = fs::read_to_string(messages).unwrap();
        let artifact = saved
            .lines()
            .find(|line| {
                let value: serde_json::Value = serde_json::from_str(line).unwrap();
                value["reason"] == "compiler-artifact"
                    && value["target"]["name"] == "holla"
                    && value["target"]["kind"] == serde_json::json!(["bin"])
            })
            .unwrap();
        let mutated = format!("{saved}{artifact}\n");
        fs::write(messages, &mutated).unwrap();
        let mut changed = manifest.clone();
        changed.messages_sha256 = hash(mutated.as_bytes());
        fs::write(&path, serde_json::to_vec(&changed).unwrap()).unwrap();
        assert!(verify(&root, &path).is_err());
        fs::remove_dir_all(directory).unwrap();
    }
}
