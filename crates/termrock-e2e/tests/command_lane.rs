mod command {
//! RUN-03 executor for this `termrock-e2e` integration-test module, adapted
//! from static-reviewed source SHA-256
//! `216c5def328d7335052c9ae1d405b4c3f2da7a3eecd586d79dff6cb8a2321077`.
//! This integration copy has not been compiled or executed.
//!
//! The executor uses pinned `termpane::session::PtySession` for a single
//! combined-output PTY stream, child status, and owned cleanup, and calls
//! Tuiscotti's public replay/render/export path for capture. It does not
//! claim parent-terminal restoration: only the child PTY's live mode/cursor
//! observations are available here.
//!
//! Integration dependencies on Unix: `termpane = { version = "=0.1.0",
//! features = ["pty"] }` and `nix = { version = "=0.31.3", default-features =
//! false, features = ["process", "signal"] }`. `termpane` brings the pinned
//! `portable-pty 0.9.0` transport; no copied terminal painter is introduced.

use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Status {
    NotRun,
    Pass,
    Fail,
    Blocked,
    Unknown,
    NotApplicable,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Layer {
    Build,
    Launch,
    FirstFrame,
    Interaction,
    Exit,
    Restoration,
    Cleanup,
    Visual,
    Ownership,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct LayerReceipt {
    pub status: Status,
    pub reason: String,
    pub evidence: Vec<String>,
}

impl LayerReceipt {
    fn not_run() -> Self {
        Self {
            status: Status::NotRun,
            reason: "no RUN-03 command has executed".into(),
            evidence: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum InvocationKind {
    ExactRootCommand,
    DirectExecutableDiagnostic,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct InvocationPlan {
    pub kind: InvocationKind,
    /// Exact argv after PATH resolution. The root-command argv stays the
    /// transcript command; the executable diagnostic has its own argv.
    pub argv: Vec<String>,
    /// Resolved from the frozen source manifest by the caller, never from a
    /// current checkout, branch name, or test working directory.
    pub cwd: PathBuf,
    pub size: (u16, u16),
    pub input: Vec<InputEvent>,
    pub max_concurrent_ptys: u8,
    pub cargo_build_jobs: u8,
    pub platform: PlatformEligibility,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum PlatformEligibility {
    SupportedLinuxOrMacos,
    BlockedUnsupportedOrUnverified,
}

pub const MAX_CONCURRENT_PTYS: u8 = 2;
pub const CARGO_BUILD_JOBS: u8 = 2;

pub fn platform_eligibility(os: &str) -> PlatformEligibility {
    match os {
        "linux" | "macos" => PlatformEligibility::SupportedLinuxOrMacos,
        _ => PlatformEligibility::BlockedUnsupportedOrUnverified,
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub enum InputEvent {
    WaitFor {
        checkpoint: &'static str,
        contains: &'static [&'static str],
        absent: &'static [&'static str],
    },
    Press(&'static str),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct InvocationReceipt {
    pub role: &'static str,
    pub kind: InvocationKind,
    pub argv: Vec<String>,
    pub cwd: PathBuf,
    pub source_manifest: PathBuf,
    pub source_manifest_sha256: String,
    pub source_commit: String,
    pub pre_command_executable_sha256: String,
    pub layers: BTreeMap<Layer, LayerReceipt>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CommandRow {
    pub id: &'static str,
    pub exact_command: &'static str,
    pub status: Status,
    pub reference: Option<InvocationReceipt>,
    pub candidate: Option<InvocationReceipt>,
    /// RUN-03 executable runs are separate diagnostics and never overwrite
    /// the exact Cargo-command result. Other command rows remain empty here.
    pub direct_executable_diagnostics: Vec<InvocationReceipt>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CommandReceipt {
    pub schema: &'static str,
    pub rows: Vec<CommandRow>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FrozenCommandSubject {
    pub role: &'static str,
    pub source_manifest: PathBuf,
    pub source_manifest_sha256: String,
    pub source_root: PathBuf,
    pub source_commit: String,
    pub toolchain: String,
    pub target_triple: String,
    pub executable: PathBuf,
    pub executable_sha256: String,
}

pub const COMMANDS: [(&str, &str, &[&str]); 7] = [
    ("RUN-01", "cargo run --release --bin showcase", &["cargo", "run", "--release", "--bin", "showcase"]),
    ("RUN-02", "cargo run --release --bin jackin-preview", &["cargo", "run", "--release", "--bin", "jackin-preview"]),
    ("RUN-03", "cargo run --release --bin holla", &["cargo", "run", "--release", "--bin", "holla"]),
    ("RUN-04", "cargo run --release --bin tablepro", &["cargo", "run", "--release", "--bin", "tablepro"]),
    ("RUN-05", "cargo run --release --bin tablepro -- --connect Production", &["cargo", "run", "--release", "--bin", "tablepro", "--", "--connect", "Production"]),
    ("RUN-06", "cargo run --release --bin jackin-preview -- --scenario accounts-mixed", &["cargo", "run", "--release", "--bin", "jackin-preview", "--", "--scenario", "accounts-mixed"]),
    ("RUN-07", "cargo run --release --bin holla -- --scenario remote-host", &["cargo", "run", "--release", "--bin", "holla", "--", "--scenario", "remote-host"]),
];

/// One immutable input/assertion program is reused for reference, candidate,
/// and the separate direct-executable diagnostic. It contains no scenario,
/// pause/motion override, fixture environment, or app configuration.
pub const RUN03_INPUT: &[InputEvent] = &[
    InputEvent::WaitFor {
        checkpoint: "00-root",
        contains: &["holla❯", "Type Search"],
        absent: &[],
    },
    InputEvent::Press("f1"),
    InputEvent::WaitFor {
        checkpoint: "01-help",
        contains: &["Key reference", "Everywhere"],
        absent: &[],
    },
    InputEvent::Press("escape"),
    InputEvent::WaitFor {
        checkpoint: "02-root-restored",
        contains: &["holla❯", "Type Search"],
        absent: &["Key reference"],
    },
    // Holla source maps Ctrl+C to immediate quit. The event is a request; it
    // cannot itself prove the child exited or the parent terminal was restored.
    InputEvent::Press("ctrl-c"),
];

pub fn initial_receipt(subjects: &[FrozenCommandSubject]) -> Result<CommandReceipt, String> {
    let reference = subjects.iter().find(|subject| subject.role == "reference");
    let candidate = subjects.iter().find(|subject| subject.role == "candidate");
    if subjects.len() != 2 || reference.is_none() || candidate.is_none() {
        return Err("receipt needs exactly one frozen reference and one frozen candidate".into());
    }
    for subject in subjects {
        if !subject.source_manifest.is_absolute()
            || !subject.source_root.is_absolute()
            || !subject.executable.is_absolute()
        {
            return Err(format!("{} paths must be absolute", subject.role));
        }
    }
    let rows = COMMANDS
        .iter()
        .map(|(id, command, argv)| {
            let invocation = |subject: &FrozenCommandSubject| {
                empty_invocation(
                    subject,
                    InvocationKind::ExactRootCommand,
                    argv.iter().map(|part| (*part).to_owned()).collect(),
                )
            };
            let diagnostics = if *id == "RUN-03" {
                subjects
                    .iter()
                    .map(|subject| {
                        let path = subject.executable.to_str().ok_or_else(|| {
                            format!("{} direct executable path is not UTF-8", subject.role)
                        })?;
                        Ok(empty_invocation(
                            subject,
                            InvocationKind::DirectExecutableDiagnostic,
                            vec![path.to_owned()],
                        ))
                    })
                    .collect::<Result<Vec<_>, String>>()?
            } else {
                Vec::new()
            };
            Ok(CommandRow {
                id,
                exact_command: command,
                status: Status::NotRun,
                reference: reference.map(invocation),
                candidate: candidate.map(invocation),
                direct_executable_diagnostics: diagnostics,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(CommandReceipt {
        schema: "termrock-spec/visibility-command-lane-v1",
        rows,
    })
}

fn empty_invocation(
    subject: &FrozenCommandSubject,
    kind: InvocationKind,
    argv: Vec<String>,
) -> InvocationReceipt {
    let layers = [
        Layer::Build,
        Layer::Launch,
        Layer::FirstFrame,
        Layer::Interaction,
        Layer::Exit,
        Layer::Restoration,
        Layer::Cleanup,
        Layer::Visual,
        Layer::Ownership,
    ]
    .into_iter()
    .map(|layer| (layer, LayerReceipt::not_run()))
    .collect();
    InvocationReceipt {
        role: subject.role,
        kind,
        argv,
        cwd: subject.source_root.clone(),
        source_manifest: subject.source_manifest.clone(),
        source_manifest_sha256: subject.source_manifest_sha256.clone(),
        source_commit: subject.source_commit.clone(),
        pre_command_executable_sha256: subject.executable_sha256.clone(),
        layers,
    }
}

pub fn run03_plans(subject: &FrozenCommandSubject) -> Result<[InvocationPlan; 2], String> {
    if subject.role != "reference" && subject.role != "candidate" {
        return Err(format!("unsupported subject role {:?}", subject.role));
    }
    if !subject.source_manifest.is_absolute()
        || !subject.source_root.is_absolute()
        || !subject.executable.is_absolute()
    {
        return Err("frozen manifest, source root, and executable paths must be absolute".into());
    }
    let root = subject.source_root.clone();
    let input = RUN03_INPUT.to_vec();
    let root_command = InvocationPlan {
        kind: InvocationKind::ExactRootCommand,
        argv: COMMANDS[2].2.iter().map(|part| (*part).to_owned()).collect(),
        cwd: root.clone(),
        size: (120, 40),
        input: input.clone(),
        max_concurrent_ptys: MAX_CONCURRENT_PTYS,
        cargo_build_jobs: CARGO_BUILD_JOBS,
        platform: platform_eligibility(std::env::consts::OS),
    };
    let direct_executable = InvocationPlan {
        kind: InvocationKind::DirectExecutableDiagnostic,
        argv: vec![subject
            .executable
            .to_str()
            .ok_or_else(|| "direct executable path is not UTF-8".to_string())?
            .to_owned()],
        cwd: root,
        size: (120, 40),
        input,
        max_concurrent_ptys: MAX_CONCURRENT_PTYS,
        cargo_build_jobs: CARGO_BUILD_JOBS,
        platform: platform_eligibility(std::env::consts::OS),
    };
    Ok([root_command, direct_executable])
}

// Concrete executor code follows the receipt/schema helpers above. It stays
// Unix-only because the pinned PTY and raw replay APIs are Unix-only.

#[cfg(unix)]
pub mod executor {
    use super::*;
    use std::collections::BTreeSet;
    use std::ffi::OsStr;
    use std::io::Read;
    use std::process::Command;
    use std::thread;

    use termpane::process::SpawnParams;
    use termpane::session::{PtySession, SessionOptions, StreamState};
    use nix::sys::signal::{self, Signal};
    use nix::unistd::{self, Pid};

    const WIDTH: u16 = 120;
    const HEIGHT: u16 = 40;
    const OUTPUT_CAP: usize = 64 * 1024 * 1024;
    const BUILD_TIMEOUT: Duration = Duration::from_secs(15 * 60);
    const CHECKPOINT_TIMEOUT: Duration = Duration::from_secs(20);
    const EXIT_TIMEOUT: Duration = Duration::from_secs(15);
    const INPUT_F1: &[u8] = b"\x1bOP";
    const INPUT_ESCAPE: &[u8] = b"\x1b";
    const INPUT_CTRL_C: &[u8] = b"\x03";

    const REQUIRED_ENV: &[&str] = &[
        "PATH",
        "HOME",
        "CARGO_HOME",
        "RUSTUP_HOME",
        "RUSTUP_TOOLCHAIN",
        "CARGO_TARGET_DIR",
        "CARGO_BUILD_JOBS",
        "CARGO_NET_OFFLINE",
        "TMPDIR",
        "TERM",
        "COLORTERM",
        "LC_ALL",
        "SHELL",
    ];

    const FORBIDDEN_ENV: &[&str] = &[
        "RUSTC",
        "RUSTC_WRAPPER",
        "RUSTC_WORKSPACE_WRAPPER",
        "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
        "CARGO_BUILD_TARGET",
        "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER",
        "CARGO_TARGET_AARCH64_APPLE_DARWIN_RUNNER",
        "MBX_LAUNCH",
    ];

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
    pub struct ArtifactRef {
        pub path: String,
        pub sha256: String,
        pub bytes: u64,
    }

    pub trait ArtifactSink {
        fn store(&mut self, relative_path: &str, bytes: &[u8]) -> Result<ArtifactRef, String>;
    }

    /// New files only; the artifact root must already be an existing external
    /// directory and must not be inside either frozen source root.
    pub struct DirectoryArtifactSink {
        root: PathBuf,
    }

    impl DirectoryArtifactSink {
        pub fn new(root: &Path, protected_roots: &[PathBuf]) -> Result<Self, String> {
            if !root.is_absolute() || !root.is_dir() {
                return Err("artifact root must be an existing absolute directory".into());
            }
            let canonical_root = fs::canonicalize(root)
                .map_err(|error| format!("canonicalize artifact root: {error}"))?;
            for protected in protected_roots {
                let canonical = fs::canonicalize(protected)
                    .map_err(|error| format!("canonicalize protected source root: {error}"))?;
                if canonical_root.starts_with(&canonical) || canonical.starts_with(&canonical_root) {
                    return Err(format!(
                        "artifact root overlaps protected source root {}",
                        canonical.display()
                    ));
                }
            }
            Ok(Self { root: canonical_root })
        }

        fn root(&self) -> &Path { &self.root }
    }

    impl ArtifactSink for DirectoryArtifactSink {
        fn store(&mut self, relative_path: &str, bytes: &[u8]) -> Result<ArtifactRef, String> {
            let relative = Path::new(relative_path);
            if relative.as_os_str().is_empty()
                || relative.components().any(|part| !matches!(part, Component::Normal(_)))
            {
                return Err(format!("unsafe artifact path {relative_path:?}"));
            }
            let path = self.root.join(relative);
            let parent = path
                .parent()
                .ok_or_else(|| "artifact file has no parent".to_string())?;
            fs::create_dir_all(parent)
                .map_err(|error| format!("create artifact directory: {error}"))?;
            let canonical_parent = fs::canonicalize(parent)
                .map_err(|error| format!("canonicalize artifact directory: {error}"))?;
            if !canonical_parent.starts_with(&self.root) {
                return Err("artifact directory escaped the validated root".into());
            }
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .map_err(|error| format!("create new artifact {}: {error}", path.display()))?;
            file.write_all(bytes)
                .map_err(|error| format!("write artifact {}: {error}", path.display()))?;
            file.sync_all()
                .map_err(|error| format!("sync artifact {}: {error}", path.display()))?;
            Ok(ArtifactRef {
                path: path.display().to_string(),
                sha256: sha256(bytes),
                bytes: bytes.len() as u64,
            })
        }
    }

    #[derive(Clone, Debug, Eq, PartialEq, Serialize)]
    pub struct ConfigReceipt {
        pub path: String,
        pub sha256: String,
        pub bytes: u64,
    }

    #[derive(Clone, Debug)]
    pub struct ExecutionEnvelope {
        /// Child variables only. `SpawnParams::env_clear` ensures nothing
        /// else is inherited; the backend's required `SHELL` is overridden.
        pub variables: BTreeMap<String, String>,
        /// The exact files resolved through `PATH`; both hashes are recorded.
        pub cargo_path: PathBuf,
        pub cargo_sha256: String,
        pub rustc_path: PathBuf,
        pub rustc_sha256: String,
        /// Where the exact `cargo run` writes `target/release/holla`.
        pub target_dir: PathBuf,
        pub artifact_root: PathBuf,
        /// Must equal two; the team Cargo slot is externally reserved.
        pub cargo_jobs: u8,
    }

    #[derive(Clone, Debug, Eq, PartialEq, Serialize)]
    pub struct EnvironmentReceipt {
        pub variable_hashes: BTreeMap<String, String>,
        pub cargo_path: String,
        pub cargo_sha256: String,
        pub rustc_path: String,
        pub rustc_sha256: String,
        pub cargo_configs: Vec<ConfigReceipt>,
        pub target_dir: String,
        pub target_triple: String,
        pub cargo_jobs: u8,
        pub inherited_environment: String,
        pub cargo_runner: String,
    }

    impl ExecutionEnvelope {
        fn validate(&self, subject: &FrozenCommandSubject, protected_roots: &[PathBuf]) -> Result<EnvironmentReceipt, String> {
            if self.cargo_jobs != CARGO_BUILD_JOBS {
                return Err(format!("CARGO_BUILD_JOBS must be {CARGO_BUILD_JOBS}"));
            }
            if !subject.source_root.is_absolute()
                || !subject.source_manifest.is_absolute()
                || !subject.executable.is_absolute()
                || !self.target_dir.is_absolute()
                || !self.artifact_root.is_absolute()
            {
                return Err("source, manifest, executable, target, and artifact paths must be absolute".into());
            }
            if !valid_sha256(&subject.source_manifest_sha256)
                || !valid_sha256(&subject.executable_sha256)
                || !valid_sha256(&self.cargo_sha256)
                || !valid_sha256(&self.rustc_sha256)
            {
                return Err("frozen manifest, executable, cargo, and rustc hashes must be SHA-256".into());
            }
            if subject.toolchain.trim().is_empty() {
                return Err("frozen source manifest must supply the active Rust toolchain identity".into());
            }
            if !native_target_matches(&subject.target_triple) {
                return Err(format!("frozen target {} does not match this native Linux/macOS host", subject.target_triple));
            }
            let actual_manifest = hash_file(&subject.source_manifest)?;
            if actual_manifest != subject.source_manifest_sha256 {
                return Err("frozen source manifest changed after handoff".into());
            }
            if hash_file(&subject.executable)? != subject.executable_sha256 {
                return Err("frozen Holla executable changed after source-pair admission".into());
            }
            if self.cargo_jobs != 2 || self.variables.get("CARGO_BUILD_JOBS").map(String::as_str) != Some("2") {
                return Err("environment must fix CARGO_BUILD_JOBS=2".into());
            }
            let actual_target = absolute_normalized(&self.target_dir)?;
            let expected_target = absolute_normalized(&subject.executable)
                .and_then(|path| path.parent().and_then(Path::parent).map(Path::to_path_buf)
                    .ok_or_else(|| "Holla executable must be under target/release".to_string()))?;
            if actual_target != expected_target
                || subject.executable.file_name() != Some(OsStr::new("holla"))
                || subject.executable.parent().and_then(Path::file_name) != Some(OsStr::new("release"))
            {
                return Err("CARGO_TARGET_DIR must resolve to the frozen target/release/holla executable".into());
            }
            for name in REQUIRED_ENV {
                let value = self.variables.get(*name).ok_or_else(|| format!("missing required environment key {name}"))?;
                if value.trim().is_empty() {
                    return Err(format!("environment key {name} must be nonempty"));
                }
            }
            let allowed: BTreeSet<&str> = REQUIRED_ENV.iter().copied().collect();
            for name in self.variables.keys() {
                if !allowed.contains(name.as_str()) || name.starts_with("HOLLA_") {
                    return Err(format!("environment key {name:?} is outside the RUN-03 allowlist"));
                }
            }
            for name in FORBIDDEN_ENV {
                if self.variables.contains_key(*name) {
                    return Err(format!("environment override {name} is forbidden for exact RUN-03"));
                }
            }
            for (name, expected) in [
                ("CARGO_NET_OFFLINE", "true"),
                ("TERM", "xterm-256color"),
                ("COLORTERM", "truecolor"),
                ("LC_ALL", "C.UTF-8"),
                ("SHELL", "/bin/sh"),
            ] {
                if self.variables.get(name).map(String::as_str) != Some(expected) {
                    return Err(format!("environment key {name} must be {expected:?}"));
                }
            }
            if self.variables.get("RUSTUP_TOOLCHAIN").map(String::as_str) != Some(subject.toolchain.as_str()) {
                return Err("RUSTUP_TOOLCHAIN differs from the frozen source build identity".into());
            }
            let path = Path::new(&self.variables["PATH"]);
            if std::env::split_paths(path.as_os_str()).any(|entry| !entry.is_absolute()) {
                return Err("every PATH entry must be absolute".into());
            }
            verify_path_tool(path, "cargo", &self.cargo_path, &self.cargo_sha256)?;
            verify_path_tool(path, "rustc", &self.rustc_path, &self.rustc_sha256)?;
            for key in ["HOME", "CARGO_HOME", "RUSTUP_HOME", "TMPDIR", "CARGO_TARGET_DIR"] {
                let path = Path::new(&self.variables[key]);
                if !path.is_absolute() {
                    return Err(format!("{key} must be an absolute path"));
                }
                let resolved = if path.exists() {
                    fs::canonicalize(path).map_err(|error| format!("canonicalize {key}: {error}"))?
                } else {
                    let parent = path.parent().ok_or_else(|| format!("{key} has no existing parent"))?;
                    fs::canonicalize(parent)
                        .map_err(|error| format!("canonicalize {key} parent: {error}"))?
                        .join(path.file_name().ok_or_else(|| format!("{key} has no final component"))?)
                };
                let mut overlaps_source = false;
                for root in protected_roots {
                    let canonical = fs::canonicalize(root)
                        .map_err(|error| format!("canonicalize protected source root: {error}"))?;
                    overlaps_source |= resolved.starts_with(&canonical) || canonical.starts_with(&resolved);
                }
                if overlaps_source {
                    return Err(format!("{key} must be outside both frozen source trees"));
                }
            }
            let env_target = absolute_normalized(Path::new(&self.variables["CARGO_TARGET_DIR"]))?;
            if env_target != actual_target {
                return Err("CARGO_TARGET_DIR variable differs from the validated target directory".into());
            }
            let artifact_root = fs::canonicalize(&self.artifact_root)
                .map_err(|error| format!("canonicalize artifact root: {error}"))?;
            if protected_roots.iter().any(|root| {
                fs::canonicalize(root).is_ok_and(|canonical| artifact_root.starts_with(&canonical) || canonical.starts_with(&artifact_root))
            }) {
                return Err("artifact root overlaps a frozen source tree".into());
            }
            let configs = inspect_cargo_configs(&subject.source_root, Path::new(&self.variables["CARGO_HOME"]))?;
            let variable_hashes = self
                .variables
                .iter()
                .map(|(name, value)| (name.clone(), sha256(value.as_bytes())))
                .collect();
            Ok(EnvironmentReceipt {
                variable_hashes,
                cargo_path: fs::canonicalize(&self.cargo_path)
                    .map_err(|error| format!("canonicalize cargo path: {error}"))?
                    .display().to_string(),
                cargo_sha256: self.cargo_sha256.clone(),
                rustc_path: fs::canonicalize(&self.rustc_path)
                    .map_err(|error| format!("canonicalize rustc path: {error}"))?
                    .display().to_string(),
                rustc_sha256: self.rustc_sha256.clone(),
                cargo_configs: configs,
                target_dir: actual_target.display().to_string(),
                target_triple: subject.target_triple.clone(),
                cargo_jobs: self.cargo_jobs,
                inherited_environment: "cleared; backend SHELL explicitly overridden".into(),
                cargo_runner: "blocked unless no target runner or compiler wrapper is configured".into(),
            })
        }
    }

    fn native_target_matches(target: &str) -> bool {
        let arch = std::env::consts::ARCH;
        match std::env::consts::OS {
            "linux" => target.starts_with(&format!("{arch}-unknown-linux-")),
            "macos" => target == format!("{arch}-apple-darwin"),
            _ => false,
        }
    }

    fn verify_path_tool(path: &Path, name: &str, expected: &Path, expected_sha: &str) -> Result<(), String> {
        let resolved = std::env::split_paths(path.as_os_str())
            .map(|entry| entry.join(name))
            .find(|candidate| candidate.is_file())
            .ok_or_else(|| format!("{name} is not resolvable from the controlled PATH"))?;
        let resolved = fs::canonicalize(resolved)
            .map_err(|error| format!("canonicalize PATH {name}: {error}"))?;
        let expected = fs::canonicalize(expected)
            .map_err(|error| format!("canonicalize pinned {name}: {error}"))?;
        if resolved != expected || hash_file(&resolved)? != expected_sha {
            return Err(format!("PATH-resolved {name} path or bytes differ from the execution envelope"));
        }
        Ok(())
    }

    fn inspect_cargo_configs(source_root: &Path, cargo_home: &Path) -> Result<Vec<ConfigReceipt>, String> {
        let mut candidate_paths = Vec::new();
        let mut current = Some(source_root);
        while let Some(dir) = current {
            let cargo_dir = dir.join(".cargo");
            for file in [cargo_dir.join("config.toml"), cargo_dir.join("config")] {
                if file.is_file() { candidate_paths.push(file); }
            }
            current = dir.parent();
        }
        for file in [cargo_home.join("config.toml"), cargo_home.join("config")] {
            if file.is_file() { candidate_paths.push(file); }
        }
        candidate_paths.sort();
        candidate_paths.dedup();
        let mut result = Vec::new();
        for path in candidate_paths {
            let bytes = fs::read(&path).map_err(|error| format!("read Cargo config {}: {error}", path.display()))?;
            let text = String::from_utf8_lossy(&bytes);
            if let Some(line_number) = first_forbidden_cargo_config_line(&text) {
                return Err(format!(
                    "Cargo config {}:{} contains a runner, wrapper, environment, rustflags, target, or source override",
                    path.display(), line_number
                ));
            }
            result.push(ConfigReceipt {
                path: fs::canonicalize(&path)
                    .map_err(|error| format!("canonicalize Cargo config: {error}"))?
                    .display().to_string(),
                sha256: sha256(&bytes),
                bytes: bytes.len() as u64,
            });
        }
        Ok(result)
    }

    fn first_forbidden_cargo_config_line(text: &str) -> Option<usize> {
        text.lines().enumerate().find_map(|(index, raw)| {
            let line = raw.split('#').next().unwrap_or_default().trim().to_ascii_lowercase();
            (line.starts_with("[env")
                || line.contains("runner")
                || line.contains("rustc-wrapper")
                || line.contains("rustdoc-wrapper")
                || line.contains("replace-with")
                // Fail closed on every TOML spelling, including dotted keys and
                // inline tables. This can reject a harmless string value that
                // mentions rustflags, which is acceptable for this controlled
                // Cargo-config envelope.
                || line.contains("rustflags")
                || line.starts_with("target-dir")
                || line.starts_with("target =")
                || line.starts_with("[source."))
                .then_some(index + 1)
        })
    }

    fn absolute_normalized(path: &Path) -> Result<PathBuf, String> {
        if !path.is_absolute() { return Err(format!("path is not absolute: {}", path.display())); }
        let mut out = PathBuf::new();
        for part in path.components() {
            match part {
                Component::RootDir | Component::Prefix(_) => out.push(part.as_os_str()),
                Component::CurDir => {}
                Component::ParentDir => { out.pop(); }
                Component::Normal(value) => out.push(value),
            }
        }
        Ok(out)
    }

    #[derive(Clone, Debug, Eq, PartialEq, Serialize)]
    pub struct CursorRecord {
        pub row: u16,
        pub col: u16,
        pub visible: bool,
        pub style: u16,
        pub text_cursor_enable: Option<bool>,
    }

    #[derive(Clone, Debug, Eq, PartialEq, Serialize)]
    pub struct DisplayModesRecord {
        pub alternate_screen: bool,
        pub application_cursor: bool,
        pub application_keypad: bool,
        pub bracketed_paste: bool,
        pub focus_events: bool,
        pub mouse_mode: String,
        pub mouse_encoding: String,
        pub kitty_keyboard: u32,
        pub in_synchronized_update: bool,
        pub mid_sequence: bool,
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct LiveSample {
        pub revision: u64,
        pub text: String,
        pub output_log: Vec<u8>,
        pub output_bytes_total: u64,
        pub output_truncated: bool,
        pub cursor: CursorRecord,
        pub modes: DisplayModesRecord,
    }

    #[derive(Clone, Debug, Eq, PartialEq, Serialize)]
    pub struct ExitRecord {
        pub success: bool,
        pub code: u32,
        pub signal_present: bool,
        pub stream: String,
    }

    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    pub enum CleanupState {
        Verified,
        Unverified,
        Failed,
    }

    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    pub struct CleanupRecord {
        pub state: CleanupState,
        pub direct_session_close: String,
        pub process_group_containment: String,
        pub signalled_pids: Vec<u32>,
        pub survivors: Vec<u32>,
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct CaptureProof {
        pub artifacts: Vec<ArtifactRef>,
        pub frame_digest: String,
        pub replay_text_matches_live: bool,
        pub replay_cursor_matches_live: bool,
    }

    pub trait CommandPty {
        fn sample(&mut self) -> Result<LiveSample, String>;
        fn wait_after(&mut self, revision: u64, deadline: Instant) -> Result<bool, String>;
        fn write_input(&mut self, bytes: &[u8]) -> Result<(), String>;
        fn has_exited(&self) -> bool;
        fn process_id(&self) -> Option<u32>;
        fn wait_exit(&mut self, deadline: Instant) -> Result<ExitRecord, String>;
        fn close(&mut self) -> CleanupRecord;
    }

    pub trait PtyFactory {
        fn spawn(
            &self,
            subject: &FrozenCommandSubject,
            plan: &InvocationPlan,
            env: &ExecutionEnvelope,
        ) -> Result<Box<dyn CommandPty>, String>;
    }

    pub trait FrameRecorder {
        fn capture(
            &mut self,
            role: &str,
            kind: InvocationKind,
            checkpoint: &str,
            sample: &LiveSample,
            recording: &tuiscotti::tui_shell::Recording,
            sink: &mut dyn ArtifactSink,
        ) -> Result<CaptureProof, String>;
    }

    pub struct TermpaneFactory;

    struct TermpanePty {
        session: PtySession,
        process_group: Option<ProcessGroupIdentity>,
        process_group_error: Option<String>,
    }

    impl PtyFactory for TermpaneFactory {
        fn spawn(
            &self,
            subject: &FrozenCommandSubject,
            plan: &InvocationPlan,
            env: &ExecutionEnvelope,
        ) -> Result<Box<dyn CommandPty>, String> {
            let mut params = SpawnParams::new(plan.argv[0].as_str())
                .args(plan.argv[1..].iter().map(String::as_str))
                .env_clear()
                .current_dir(&subject.source_root);
            for (name, value) in &env.variables {
                params = params.env(name.as_str(), value.as_str());
            }
            let session = PtySession::spawn(
                &params,
                SessionOptions {
                    cols: WIDTH,
                    rows: HEIGHT,
                    term: env.variables["TERM"].clone(),
                    colorterm: env.variables["COLORTERM"].clone(),
                    scrollback: 1000,
                    record_output: true,
                    event_cap: 128,
                    output_cap_bytes: OUTPUT_CAP,
                },
            )
            .map_err(|error| format!("spawn command PTY: {error}"))?;
            let (process_group, process_group_error) = match session.process_id() {
                Some(pid) => match capture_process_group(pid) {
                    Ok(identity) => (Some(identity), None),
                    Err(error) => (None, Some(error)),
                },
                None => (None, Some("termpane did not report the direct child PID".into())),
            };
            Ok(Box::new(TermpanePty { session, process_group, process_group_error }))
        }
    }

    impl CommandPty for TermpanePty {
        fn sample(&mut self) -> Result<LiveSample, String> {
            // The live grid and byte log are requested until their byte counts
            // agree. This prevents replaying bytes newer than the captured
            // live observation (or comparing against an older screen).
            for _ in 0..8 {
                let observation = self.session.observe().map_err(|error| format!("PTY observe: {error}"))?;
                let output = self.session.output_log().map_err(|error| format!("PTY output log: {error}"))?;
                if observation.diagnostics.output_truncated {
                    return Ok(sample_from(&observation, output));
                }
                if output.len() as u64 == observation.diagnostics.output_bytes_kept {
                    return Ok(sample_from(&observation, output));
                }
            }
            Err("could not obtain a byte-consistent live PTY observation".into())
        }

        fn wait_after(&mut self, revision: u64, deadline: Instant) -> Result<bool, String> {
            let target = revision.saturating_add(1);
            match self.session.wait_revision(target, deadline) {
                Ok(_) => Ok(true),
                Err(termpane::session::ProcessError::Timeout(_)) => Ok(false),
                Err(error) => Err(format!("PTY wait for output: {error}")),
            }
        }

        fn write_input(&mut self, bytes: &[u8]) -> Result<(), String> {
            self.session.write_stdin(bytes).map_err(|error| format!("PTY input write: {error}"))
        }

        fn has_exited(&self) -> bool { self.session.poll_exit().is_some() }

        fn process_id(&self) -> Option<u32> { self.session.process_id() }

        fn wait_exit(&mut self, deadline: Instant) -> Result<ExitRecord, String> {
            let outcome = self.session.wait_outcome(deadline)
                .map_err(|error| format!("PTY child exit wait: {error}"))?;
            Ok(ExitRecord {
                success: outcome.exit.success(),
                code: outcome.exit.exit_code(),
                signal_present: outcome.exit.signal().is_some(),
                stream: match outcome.stream {
                    StreamState::CleanEof => "clean_eof",
                    StreamState::Streaming => "streaming",
                    StreamState::ReadFailed(_) => "read_failed",
                    StreamState::DrainExpired => "drain_expired",
                    StreamState::Aborted => "aborted",
                }.to_string(),
            })
        }

        fn close(&mut self) -> CleanupRecord {
            let sweep = self.process_group.as_ref().map(sweep_process_group);
            let (close_ok, direct) = match self.session.close() {
                Ok(()) => (true, "termpane close completed and joined PTY I/O workers".to_string()),
                Err(error) => (false, format!("termpane close failed: {error}")),
            };
            let (group_state, group_reason, signalled, survivors) = match sweep {
                Some(report) => (report.state, report.reason, report.signalled, report.survivors),
                None => (CleanupState::Unverified,
                    self.process_group_error.clone().unwrap_or_else(|| "process-group identity unavailable".into()),
                    Vec::new(), Vec::new()),
            };
            let state = if !close_ok {
                CleanupState::Failed
            } else {
                group_state.clone()
            };
            CleanupRecord {
                state,
                direct_session_close: direct,
                process_group_containment: group_reason,
                signalled_pids: signalled,
                survivors,
            }
        }
    }

    #[derive(Clone, Debug)]
    struct ProcessGroupIdentity {
        pid: u32,
        pgid: i32,
        sid: i32,
        start: String,
    }

    #[derive(Clone, Debug)]
    struct ProcessRow {
        pid: u32,
        pgid: i32,
        sid: i32,
        start: String,
    }

    struct ProcessGroupSweep {
        state: CleanupState,
        reason: String,
        signalled: Vec<u32>,
        survivors: Vec<u32>,
    }

    const MAX_PROCESS_ROWS: usize = 131_072;
    const MAX_GROUP_SIGNALS: usize = 256;
    const GROUP_SETTLE: Duration = Duration::from_millis(750);

    fn capture_process_group(raw_pid: u32) -> Result<ProcessGroupIdentity, String> {
        let raw_pid = i32::try_from(raw_pid).map_err(|_| "child pid exceeds nix pid range")?;
        if raw_pid <= 1 || raw_pid == unistd::getpid().as_raw() {
            return Err("refusing unsafe child pid".into());
        }
        let pid = Pid::from_raw(raw_pid);
        let pgid = unistd::getpgid(Some(pid)).map_err(|error| format!("read child pgid: {error}"))?;
        let sid = unistd::getsid(Some(pid)).map_err(|error| format!("read child sid: {error}"))?;
        let own_pgid = unistd::getpgid(None).map_err(|error| format!("read harness pgid: {error}"))?;
        if pgid.as_raw() <= 1 || sid.as_raw() <= 1 || pgid == own_pgid || pgid.as_raw() != raw_pid || sid.as_raw() != raw_pid {
            return Err("PTY child is not a distinct session and process-group leader".into());
        }
        Ok(ProcessGroupIdentity {
            pid: raw_pid as u32,
            pgid: pgid.as_raw(),
            sid: sid.as_raw(),
            start: process_start_token(raw_pid as u32)?,
        })
    }

    fn process_start_token(pid: u32) -> Result<String, String> {
        process_identity(pid).map(|row| row.start)
    }

    fn ps_command(args: &[&str]) -> Result<std::process::Output, String> {
        let path = if Path::new("/bin/ps").is_file() { "/bin/ps" } else { "/usr/bin/ps" };
        let output = Command::new(path).env_clear().env("LC_ALL", "C").args(args).output()
            .map_err(|error| format!("launch process identity probe: {error}"))?;
        if !output.status.success() || output.stdout.len() > 8 * 1024 * 1024 {
            return Err("process identity probe failed or exceeded its 8 MiB bound".into());
        }
        Ok(output)
    }

    fn process_snapshot() -> Result<Vec<ProcessRow>, String> {
        let output = ps_command(&["-ax", "-o", "pid=,pgid=,sess=,lstart="])?;
        let text = String::from_utf8_lossy(&output.stdout);
        let mut rows = Vec::new();
        for line in text.lines().take(MAX_PROCESS_ROWS) {
            let mut fields = line.split_whitespace();
            let (Some(pid), Some(pgid), Some(sid)) = (fields.next(), fields.next(), fields.next()) else { continue; };
            let (Ok(pid), Ok(pgid), Ok(sid)) = (pid.parse::<u32>(), pgid.parse::<i32>(), sid.parse::<i32>()) else { continue; };
            let start = fields.collect::<Vec<_>>().join(" ");
            if !start.is_empty() { rows.push(ProcessRow { pid, pgid, sid, start }); }
        }
        if rows.len() >= MAX_PROCESS_ROWS {
            return Err("process snapshot reached its row bound".into());
        }
        Ok(rows)
    }

    fn process_identity(pid: u32) -> Result<ProcessRow, String> {
        let pid_string = pid.to_string();
        let output = ps_command(&["-o", "pid=,pgid=,sess=,lstart=", "-p", &pid_string])?;
        let line = String::from_utf8_lossy(&output.stdout);
        let mut fields = line.split_whitespace();
        let (Some(pid), Some(pgid), Some(sid)) = (fields.next(), fields.next(), fields.next()) else {
            return Err("process identity disappeared before verification".into());
        };
        let (pid, pgid, sid) = (pid.parse::<u32>(), pgid.parse::<i32>(), sid.parse::<i32>());
        let (Ok(pid), Ok(pgid), Ok(sid)) = (pid, pgid, sid) else { return Err("invalid process identity fields".into()); };
        let start = fields.collect::<Vec<_>>().join(" ");
        if start.is_empty() { return Err("process identity has no start token".into()); }
        Ok(ProcessRow { pid, pgid, sid, start })
    }

    fn sweep_process_group(identity: &ProcessGroupIdentity) -> ProcessGroupSweep {
        let own_pid = unistd::getpid().as_raw() as u32;
        let mut signalled = Vec::new();
        let initial = match process_snapshot() {
            Ok(rows) => rows,
            Err(error) => return ProcessGroupSweep { state: CleanupState::Unverified, reason: error, signalled, survivors: Vec::new() },
        };
        let members: Vec<ProcessRow> = initial.into_iter().filter(|row| row.pgid == identity.pgid).collect();
        if members.iter().any(|row| row.sid != identity.sid) {
            return ProcessGroupSweep { state: CleanupState::Unverified,
                reason: "process-group ID contains a foreign session; refusing to signal reused IDs".into(), signalled, survivors: members.iter().map(|row| row.pid).collect() };
        }
        if let Some(leader) = members.iter().find(|row| row.pid == identity.pid)
            && leader.start != identity.start
        {
            return ProcessGroupSweep { state: CleanupState::Unverified,
                reason: "child PID was reused; refusing to signal the new process group".into(), signalled, survivors: members.iter().map(|row| row.pid).collect() };
        }
        for row in members.iter().take(MAX_GROUP_SIGNALS) {
            if row.pid <= 1 || row.pid == own_pid {
                return ProcessGroupSweep { state: CleanupState::Unverified,
                    reason: "unsafe process appeared in the child process group".into(), signalled, survivors: members.iter().map(|member| member.pid).collect() };
            }
            let current = match process_identity(row.pid) {
                Ok(current) => current,
                Err(_) => continue,
            };
            if current.pgid != identity.pgid || current.sid != identity.sid || current.start != row.start {
                continue;
            }
            if row.pid == identity.pid && current.start != identity.start { continue; }
            let pid = match i32::try_from(row.pid) {
                Ok(pid) => Pid::from_raw(pid),
                Err(_) => continue,
            };
            match signal::kill(pid, Signal::SIGKILL) {
                Ok(()) => signalled.push(row.pid),
                Err(nix::errno::Errno::ESRCH) => {}
                Err(error) => {
                    return ProcessGroupSweep { state: CleanupState::Unverified,
                        reason: format!("could not signal verified child-group pid {}: {error}", row.pid),
                        signalled, survivors: members.iter().map(|member| member.pid).collect() };
                }
            }
        }
        let deadline = Instant::now() + GROUP_SETTLE;
        loop {
            let current = match process_snapshot() {
                Ok(rows) => rows,
                Err(error) => return ProcessGroupSweep { state: CleanupState::Unverified, reason: error, signalled, survivors: Vec::new() },
            };
            let survivors: Vec<u32> = current.iter()
                .filter(|row| row.pgid == identity.pgid && row.sid == identity.sid && row.pid > 1 && row.pid != own_pid)
                .map(|row| row.pid).collect();
            if survivors.is_empty() {
                return ProcessGroupSweep { state: CleanupState::Verified,
                    reason: "verified child process group is empty; descendants that create a new session are outside this containment boundary".into(),
                    signalled, survivors };
            }
            if Instant::now() >= deadline {
                return ProcessGroupSweep { state: CleanupState::Unverified,
                    reason: "verified group members survived the bounded 750 ms cleanup sweep".into(),
                    signalled, survivors };
            }
            thread::sleep(Duration::from_millis(25));
        }
    }

    fn sample_from(observation: &termpane::session::Observation, output: Vec<u8>) -> LiveSample {
        LiveSample {
            revision: observation.revision,
            text: observation.grid.to_text(),
            output_bytes_total: observation.diagnostics.output_bytes_total,
            output_truncated: observation.diagnostics.output_truncated,
            output_log: output,
            cursor: CursorRecord {
                row: observation.cursor.position.0,
                col: observation.cursor.position.1,
                visible: observation.cursor.visible,
                style: observation.cursor.style,
                text_cursor_enable: observation.cursor.text_cursor_enable,
            },
            modes: DisplayModesRecord {
                alternate_screen: observation.modes.alternate_screen,
                application_cursor: observation.modes.application_cursor,
                application_keypad: observation.modes.application_keypad,
                bracketed_paste: observation.modes.bracketed_paste,
                focus_events: observation.modes.focus_events,
                mouse_mode: format!("{:?}", observation.modes.mouse_mode),
                mouse_encoding: format!("{:?}", observation.modes.mouse_encoding),
                kitty_keyboard: observation.modes.kitty_keyboard,
                in_synchronized_update: observation.modes.in_synchronized_update,
                mid_sequence: observation.modes.mid_sequence,
            },
        }
    }

    pub struct TuiscottiRecorder;

    impl FrameRecorder for TuiscottiRecorder {
        fn capture(
            &mut self,
            role: &str,
            kind: InvocationKind,
            checkpoint: &str,
            sample: &LiveSample,
            recording: &tuiscotti::tui_shell::Recording,
            sink: &mut dyn ArtifactSink,
        ) -> Result<CaptureProof, String> {
            use tuiscotti::formats::capture_all;
            use tuiscotti::profile::RenderProfile;
            use tuiscotti::render::Renderer;

            let replay = tuiscotti::tui_shell::replay_recording(recording, None)
                .map_err(|error| format!("replay RUN-03 PTY output: {error}"))?;
            let frame = tuiscotti::render::frame_from_screen(&replay.screen, "default");
            let replay_text = frame.text();
            let text_matches = replay_text == sample.text;
            let cursor_matches = frame.cursor.x == sample.cursor.col
                && frame.cursor.y == sample.cursor.row
                && frame.cursor.visible == sample.cursor.visible;
            let profile = RenderProfile::vendored();
            let mut renderer = Renderer::for_render_profile(&profile)
                .map_err(|error| format!("initialize strict Tuiscotti renderer: {error}"))?;
            let bundle = capture_all(&mut renderer, &frame, &format!("RUN-03:{checkpoint}"))
                .map_err(|error| format!("capture RUN-03 checkpoint: {error}"))?;
            let prefix = format!("RUN-03/{role}/{}/{}", invocation_slug(kind), checkpoint);
            let frame_digest = format!("{:016x}", frame.digest());
            let meta = serde_json::to_vec_pretty(&serde_json::json!({
                "schema": "termrock-spec/run03-live-frame-capture-v1",
                "checkpoint": checkpoint,
                "live_revision": sample.revision,
                "live_cursor": &sample.cursor,
                "live_modes": &sample.modes,
                "live_output_bytes_total": sample.output_bytes_total,
                "live_output_log_bytes": sample.output_log.len(),
                "live_output_truncated": sample.output_truncated,
                "frame_cursor": &frame.cursor,
                "frame_digest": &frame_digest,
                "replay_bytes": replay.bytes_fed,
                "replay_chunks": replay.chunks,
                "live_text_matches_replay": text_matches,
                "live_cursor_matches_replay": cursor_matches,
                "live_text_sha256": sha256(sample.text.as_bytes()),
                "replay_text_sha256": sha256(replay_text.as_bytes()),
                "visual_comparison": "BLOCKED-no-shared-approved-generation"
            }))
            .map_err(|error| format!("serialize RUN-03 frame metadata: {error}"))?;
            let ascii_loss = serde_json::to_vec_pretty(&serde_json::json!({
                "lossy": bundle.ascii.lossy(),
                "substitutions": bundle.ascii.substitutions.iter().map(|item| serde_json::json!({
                    "x": item.x,
                    "y": item.y,
                    "original": item.original,
                    "replacement": item.replacement
                })).collect::<Vec<_>>()
            })).map_err(|error| format!("serialize ASCII loss record: {error}"))?;
            let artifacts = vec![
                sink.store(&format!("{prefix}/live-text.txt"), sample.text.as_bytes())?,
                sink.store(&format!("{prefix}/frame.json"), bundle.json.as_bytes())?,
                sink.store(&format!("{prefix}/ansi.txt"), bundle.ansi.as_bytes())?,
                sink.store(&format!("{prefix}/html.html"), bundle.html.as_bytes())?,
                sink.store(&format!("{prefix}/screen.png"), &bundle.png)?,
                sink.store(&format!("{prefix}/ascii.txt"), bundle.ascii.text.as_bytes())?,
                sink.store(&format!("{prefix}/screen.txt"), bundle.txt.as_bytes())?,
                sink.store(&format!("{prefix}/ascii-loss.json"), &ascii_loss)?,
                sink.store(&format!("{prefix}/capture.json"), &meta)?,
            ];
            Ok(CaptureProof {
                artifacts,
                frame_digest,
                replay_text_matches_live: text_matches,
                replay_cursor_matches_live: cursor_matches,
            })
        }
    }

    #[derive(Clone, Debug, Eq, PartialEq, Serialize)]
    pub struct InputReceipt {
        pub sequence: u8,
        pub key: String,
        pub bytes_hex: String,
        pub attempted: bool,
        pub write_status: String,
    }

    #[derive(Clone, Debug, Eq, PartialEq, Serialize)]
    pub struct CheckpointReceipt {
        pub id: String,
        pub status: Status,
        pub reason: String,
        pub live_text_sha256: Option<String>,
        pub live_cursor: Option<CursorRecord>,
        pub live_modes: Option<DisplayModesRecord>,
        pub capture_artifacts: Vec<ArtifactRef>,
        pub frame_digest: Option<String>,
    }

    #[derive(Clone, Debug, Eq, PartialEq, Serialize)]
    pub struct RestorationReceipt {
        pub status: Status,
        pub reason: String,
        pub before_ctrl_c: Option<DisplayModesRecord>,
        pub after_exit: Option<DisplayModesRecord>,
        pub before_ctrl_c_cursor: Option<CursorRecord>,
        pub after_exit_cursor: Option<CursorRecord>,
        pub host_terminal_state: String,
        pub child_pty_state: String,
    }

    #[derive(Clone, Debug, Eq, PartialEq, Serialize)]
    pub struct RunEvidence {
        pub invocation: InvocationReceipt,
        pub environment: Option<EnvironmentReceipt>,
        pub pre_spawn_executable_sha256: Option<String>,
        pub post_run_executable_sha256: Option<String>,
        pub checkpoints: Vec<CheckpointReceipt>,
        pub inputs: Vec<InputReceipt>,
        pub exit: Option<ExitRecord>,
        pub restoration: RestorationReceipt,
        pub artifacts: Vec<ArtifactRef>,
        pub cleanup: CleanupRecord,
    }

    struct OutputTracker {
        recorded_output_bytes: usize,
        finished_boundary: Option<usize>,
        running_boundary: Option<usize>,
        recording: tuiscotti::tui_shell::Recording,
    }

    impl OutputTracker {
        fn new(kind: InvocationKind, size: (u16, u16)) -> Self {
            Self {
                recorded_output_bytes: 0,
                finished_boundary: None,
                running_boundary: (kind == InvocationKind::DirectExecutableDiagnostic).then_some(0),
                recording: tuiscotti::tui_shell::Recording::new(size.0, size.1),
            }
        }

        fn update(
            &mut self,
            kind: InvocationKind,
            executable: &Path,
            sample: &LiveSample,
        ) -> Result<Option<usize>, String> {
            if sample.output_truncated {
                return Err("PTY output log reached its 64 MiB cap".into());
            }
            if sample.output_bytes_total != sample.output_log.len() as u64 {
                return Err("PTY output bytes are incomplete or not byte-consistent".into());
            }
            if let Some(offset) = self.running_boundary {
                if sample.output_log.len() < self.recorded_output_bytes {
                    return Err("PTY output log shrank during capture".into());
                }
                let start = self.recorded_output_bytes.max(offset);
                if sample.output_log.len() > start {
                    self.recording
                        .push_output(&sample.output_log[start..])
                        .map_err(|error| format!("record PTY output for Tuiscotti replay: {error}"))?;
                }
                self.recorded_output_bytes = sample.output_log.len();
                return Ok(self.running_boundary);
            }
            if kind == InvocationKind::ExactRootCommand {
                if self.finished_boundary.is_none() {
                    self.finished_boundary = cargo_finished_boundary(&sample.output_log)?;
                }
                match cargo_running_boundary(&sample.output_log, executable, self.finished_boundary)? {
                    Some(boundary) => {
                        self.running_boundary = Some(boundary);
                        let start = self.recorded_output_bytes.max(boundary);
                        if sample.output_log.len() > start {
                            self.recording
                                .push_output(&sample.output_log[start..])
                                .map_err(|error| format!("record PTY output for Tuiscotti replay: {error}"))?;
                        }
                        self.recorded_output_bytes = sample.output_log.len();
                    }
                    None => self.recorded_output_bytes = sample.output_log.len(),
                }
            }
            Ok(self.running_boundary)
        }

        fn record_input(&mut self, bytes: &[u8]) -> Result<(), String> {
            self.recording
                .push_input(bytes)
                .map_err(|error| format!("record PTY input for Tuiscotti replay: {error}"))
        }
    }

    fn cargo_running_boundary(
        output: &[u8],
        expected: &Path,
        finished_boundary: Option<usize>,
    ) -> Result<Option<usize>, String> {
        let expected = expected.to_string_lossy();
        let mut offset = 0usize;
        for line in output.split_inclusive(|byte| *byte == b'\n') {
            let terminated = line.ends_with(b"\n");
            let plain = strip_csi(line);
            let plain = String::from_utf8_lossy(&plain);
            let body = plain.strip_suffix('\n').unwrap_or(&plain);
            let body = body.strip_suffix('\r').unwrap_or(body);
            let body = body.trim_start();
            if body.starts_with("Running ") {
                if !terminated {
                    return Ok(None);
                }
                let marker = format!("Running `{expected}`");
                if finished_boundary.is_none_or(|finished| offset < finished) {
                    if body == marker {
                        return Err("expected Holla Running marker appeared before the release Finished boundary".into());
                    }
                    // Cargo runs build scripts before its final Finished line.
                    // Their child commands also produce Running lines; ignore
                    // those pre-build-completion lines, while rejecting an
                    // early line that claims this exact Holla executable.
                    offset += line.len();
                    continue;
                }
                if body != marker {
                    return Err(format!("post-Finished Cargo Running line is not the exact expected executable marker: {body:?}"));
                }
                return Ok(Some(offset + line.len()));
            }
            offset += line.len();
        }
        Ok(None)
    }

    fn cargo_finished_boundary(output: &[u8]) -> Result<Option<usize>, String> {
        let mut offset = 0usize;
        for line in output.split_inclusive(|byte| *byte == b'\n') {
            let terminated = line.ends_with(b"\n");
            let plain = strip_csi(line);
            let plain = String::from_utf8_lossy(&plain);
            let body = plain.strip_suffix('\n').unwrap_or(&plain);
            let body = body.strip_suffix('\r').unwrap_or(body);
            let body = body.trim_start();
            let release_profile = body.starts_with("Finished `release` profile [")
                || body.starts_with("Finished release profile [");
            if release_profile && body.contains("target(s) in ") {
                if !terminated {
                    return Ok(None);
                }
                return Ok(Some(offset + line.len()));
            }
            offset += line.len();
        }
        Ok(None)
    }

    fn strip_csi(bytes: &[u8]) -> Vec<u8> {
        let mut out = Vec::with_capacity(bytes.len());
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == 0x1b && bytes.get(i + 1) == Some(&b'[') {
                i += 2;
                while i < bytes.len() {
                    let byte = bytes[i];
                    i += 1;
                    if (0x40..=0x7e).contains(&byte) { break; }
                }
            } else {
                out.push(bytes[i]);
                i += 1;
            }
        }
        out
    }

    fn checkpoint_matches(plan: &InvocationPlan, id: &str, text: &str) -> bool {
        plan.input.iter().find_map(|event| match event {
            InputEvent::WaitFor { checkpoint, contains, absent } if *checkpoint == id => {
                Some(contains.iter().all(|needle| text.contains(*needle))
                    && absent.iter().all(|needle| !text.contains(*needle)))
            }
            _ => None,
        }).unwrap_or(false)
    }

    fn wait_checkpoint(
        port: &mut dyn CommandPty,
        tracker: &mut OutputTracker,
        plan: &InvocationPlan,
        kind: InvocationKind,
        executable: &Path,
        id: &str,
        deadline: Instant,
    ) -> Result<(LiveSample, usize), String> {
        loop {
            let sample = port.sample()?;
            let boundary = tracker.update(kind, executable, &sample)?;
            if boundary.is_some() && checkpoint_matches(plan, id, &sample.text) {
                return Ok((sample, boundary.expect("checked boundary")));
            }
            if port.has_exited() {
                return Err(format!("child exited before checkpoint {id}"));
            }
            if Instant::now() >= deadline {
                return Err(format!("checkpoint {id} timed out"));
            }
            if !port.wait_after(sample.revision, deadline)? {
                return Err(format!("checkpoint {id} timed out"));
            }
        }
    }

    fn record_checkpoint(
        role: &str,
        kind: InvocationKind,
        id: &str,
        sample: LiveSample,
        tracker: &OutputTracker,
        recorder: &mut dyn FrameRecorder,
        sink: &mut dyn ArtifactSink,
    ) -> CheckpointReceipt {
        let capture = recorder.capture(role, kind, id, &sample, &tracker.recording, sink);
        match capture {
            Ok(proof) => {
                let status = if proof.replay_text_matches_live && proof.replay_cursor_matches_live {
                    Status::Pass
                } else {
                    Status::Fail
                };
                let reason = if status == Status::Pass {
                    "live termpane observation matches the Tuiscotti replay frame"
                } else {
                    "live observation differs from replay text or cursor; capture retained"
                };
                CheckpointReceipt {
                    id: id.to_string(),
                    status,
                    reason: reason.into(),
                    live_text_sha256: Some(sha256(sample.text.as_bytes())),
                    live_cursor: Some(sample.cursor),
                    live_modes: Some(sample.modes),
                    capture_artifacts: proof.artifacts,
                    frame_digest: Some(proof.frame_digest),
                }
            }
            Err(error) => CheckpointReceipt {
                id: id.to_string(),
                status: Status::Fail,
                reason: format!("checkpoint capture failed: {error}"),
                live_text_sha256: Some(sha256(sample.text.as_bytes())),
                live_cursor: Some(sample.cursor),
                live_modes: Some(sample.modes),
                capture_artifacts: Vec::new(),
                frame_digest: None,
            },
        }
    }

    fn send_key(
        port: &mut dyn CommandPty,
        tracker: &mut OutputTracker,
        sequence: u8,
        key: &str,
        bytes: &[u8],
        inputs: &mut Vec<InputReceipt>,
    ) -> bool {
        if port.has_exited() {
            inputs.push(input_record(sequence, key, bytes, false, "child_already_exited"));
            return false;
        }
        match port.write_input(bytes) {
            Ok(()) => {
                match tracker.record_input(bytes) {
                    Ok(()) => {
                        inputs.push(input_record(sequence, key, bytes, true, "written_and_recorded"));
                        true
                    }
                    Err(error) => {
                        inputs.push(input_record(sequence, key, bytes, true, &format!("written_but_record_failed:{error}")));
                        false
                    }
                }
            }
            Err(error) => {
                inputs.push(input_record(sequence, key, bytes, true, &format!("write_failed:{error}")));
                false
            }
        }
    }

    fn planned_key_bytes(plan: &InvocationPlan, requested: &str) -> Option<&'static [u8]> {
        let is_declared = plan.input.iter().any(|event| {
            matches!(event, InputEvent::Press(key) if *key == requested)
        });
        if !is_declared { return None; }
        match requested {
            "f1" => Some(INPUT_F1),
            "escape" => Some(INPUT_ESCAPE),
            "ctrl-c" => Some(INPUT_CTRL_C),
            _ => None,
        }
    }

    fn send_planned_key(
        port: &mut dyn CommandPty,
        tracker: &mut OutputTracker,
        plan: &InvocationPlan,
        sequence: u8,
        key: &str,
        inputs: &mut Vec<InputReceipt>,
    ) -> bool {
        match planned_key_bytes(plan, key) {
            Some(bytes) => send_key(port, tracker, sequence, key, bytes, inputs),
            None => {
                inputs.push(input_record(sequence, key, &[], false, "input_not_in_frozen_plan"));
                false
            }
        }
    }

    fn input_record(sequence: u8, key: &str, bytes: &[u8], attempted: bool, status: &str) -> InputReceipt {
        InputReceipt {
            sequence,
            key: key.to_string(),
            bytes_hex: bytes.iter().map(|byte| format!("{byte:02x}")).collect(),
            attempted,
            write_status: status.to_string(),
        }
    }

    fn drive_invocation(
        subject: &FrozenCommandSubject,
        plan: &InvocationPlan,
        environment: Option<EnvironmentReceipt>,
        mut port: Box<dyn CommandPty>,
        recorder: &mut dyn FrameRecorder,
        sink: &mut dyn ArtifactSink,
    ) -> RunEvidence {
        let mut invocation = empty_invocation(subject, plan.kind, plan.argv.clone());
        let mut tracker = OutputTracker::new(plan.kind, plan.size);
        let mut checkpoints = Vec::new();
        let mut inputs = Vec::new();
        let mut artifact_refs = Vec::new();
        let mut interaction_ok = true;
        let mut first_frame = Status::Blocked;
        let mut pre_ctrl_c_modes = None;
        let mut after_exit_modes = None;
        let mut pre_ctrl_c_cursor = None;
        let mut after_exit_cursor = None;
        let started_pid = port.process_id();
        if started_pid.is_some() && plan.kind == InvocationKind::DirectExecutableDiagnostic {
            set_layer(&mut invocation, Layer::Launch, Status::Pass, "PTY child process was created", vec![format!("pid={}", started_pid.unwrap())]);
        } else if started_pid.is_none() {
            set_layer(&mut invocation, Layer::Launch, Status::Fail, "PTY backend did not report the direct child PID", vec![]);
        }

        if plan.kind == InvocationKind::DirectExecutableDiagnostic {
            set_layer(&mut invocation, Layer::Build, Status::NotApplicable,
                "separate direct-executable diagnostic; exact Cargo build result remains in the root-command invocation", vec![]);
        }

        let root_deadline = Instant::now() + BUILD_TIMEOUT;
        let build_ready = if plan.kind == InvocationKind::ExactRootCommand {
            let mut build_recorded = false;
            loop {
                let sample = match port.sample() {
                    Ok(sample) => sample,
                    Err(error) => {
                        if invocation.layers.get(&Layer::Build).is_none_or(|layer| layer.status != Status::Pass) {
                            set_layer(&mut invocation, Layer::Build, Status::Fail,
                                format!("read Cargo PTY build output failed: {error}"), vec![]);
                        }
                        set_layer(&mut invocation, Layer::Launch, Status::Fail,
                            "no verified Cargo Running marker was observed", vec![]);
                        break false;
                    }
                };
                let running = tracker.update(plan.kind, &subject.executable, &sample);
                if !build_recorded {
                    if let Some(boundary) = tracker.finished_boundary.filter(|boundary| *boundary <= sample.output_log.len()) {
                        build_recorded = true;
                        match sink.store(
                            &format!("RUN-03/{}/{}/build.raw", subject.role, invocation_slug(plan.kind)),
                            &sample.output_log[..boundary],
                        ) {
                            Ok(artifact) => {
                                let digest = artifact.sha256.clone();
                                artifact_refs.push(artifact);
                                set_layer(&mut invocation, Layer::Build, Status::Pass,
                                    "Cargo emitted a complete Finished line; the root build result is recorded separately from launch",
                                    vec![format!("finished_boundary={boundary}"), digest]);
                            }
                            Err(error) => set_layer(&mut invocation, Layer::Build, Status::Fail,
                                format!("Cargo Finished marker was observed but its build artifact failed: {error}"), vec![]),
                        }
                    }
                }
                match running {
                    Ok(Some(boundary)) => {
                        if invocation.layers.get(&Layer::Build).is_none_or(|layer| layer.status == Status::NotRun) {
                            set_layer(&mut invocation, Layer::Build, Status::Fail,
                                "Cargo emitted a Running line without a complete Finished line", vec![format!("running_boundary={boundary}")]);
                        }
                        if started_pid.is_some() {
                            set_layer(&mut invocation, Layer::Launch, Status::Pass,
                                "Cargo emitted a complete Running line naming the frozen release executable",
                                vec![format!("pid={}", started_pid.unwrap()), format!("running_boundary={boundary}")]);
                        } else {
                            set_layer(&mut invocation, Layer::Launch, Status::Fail,
                                "Cargo Running line was seen but PTY did not report its direct child PID", vec![]);
                        }
                        break true;
                    }
                    Ok(None) => {
                        if port.has_exited() {
                            if invocation.layers.get(&Layer::Build).is_none_or(|layer| layer.status == Status::NotRun) {
                                set_layer(&mut invocation, Layer::Build, Status::Fail,
                                    "Cargo exited before a complete Finished line", vec![]);
                            }
                            set_layer(&mut invocation, Layer::Launch, Status::Fail,
                                "Cargo exited before a verified Running line for the frozen executable", vec![]);
                            break false;
                        }
                        if Instant::now() >= root_deadline {
                            if invocation.layers.get(&Layer::Build).is_none_or(|layer| layer.status == Status::NotRun) {
                                set_layer(&mut invocation, Layer::Build, Status::Fail,
                                    "Cargo build did not emit a complete Finished line before timeout", vec![]);
                            }
                            set_layer(&mut invocation, Layer::Launch, Status::Fail,
                                "Cargo Running marker timed out", vec![]);
                            break false;
                        }
                        if let Err(error) = port.wait_after(sample.revision, root_deadline) {
                            if invocation.layers.get(&Layer::Build).is_none_or(|layer| layer.status == Status::NotRun) {
                                set_layer(&mut invocation, Layer::Build, Status::Fail,
                                    format!("wait for Cargo Finished marker failed: {error}"), vec![]);
                            }
                            set_layer(&mut invocation, Layer::Launch, Status::Fail,
                                format!("wait for Cargo Running marker failed: {error}"), vec![]);
                            break false;
                        }
                    }
                    Err(error) => {
                        if invocation.layers.get(&Layer::Build).is_none_or(|layer| layer.status == Status::NotRun) {
                            set_layer(&mut invocation, Layer::Build, Status::Fail, error.clone(), vec![]);
                        }
                        set_layer(&mut invocation, Layer::Launch, Status::Fail,
                            format!("Cargo launch marker validation failed: {error}"), vec![]);
                        break false;
                    }
                }
            }
        } else {
            true
        };

        let mut root_checkpoint_ok = false;
        let mut help_checkpoint_ok = false;
        let mut restored_checkpoint_ok = false;
        if build_ready && started_pid.is_some() {
            let first_deadline = Instant::now() + CHECKPOINT_TIMEOUT;
            match wait_checkpoint(&mut *port, &mut tracker, plan, plan.kind, &subject.executable,
                "00-root", first_deadline) {
                Ok((sample, _)) => {
                    pre_ctrl_c_modes = Some(sample.modes.clone());
                    pre_ctrl_c_cursor = Some(sample.cursor.clone());
                    root_checkpoint_ok = true;
                    first_frame = Status::Pass;
                    checkpoints.push(record_checkpoint(subject.role, plan.kind, "00-root", sample,
                        &tracker, recorder, sink));
                    let sent = send_planned_key(&mut *port, &mut tracker, plan, 1, "f1", &mut inputs);
                    interaction_ok &= sent;
                    if sent {
                        match wait_checkpoint(&mut *port, &mut tracker, plan, plan.kind, &subject.executable,
                            "01-help", Instant::now() + CHECKPOINT_TIMEOUT) {
                            Ok((sample, _)) => {
                                help_checkpoint_ok = true;
                                checkpoints.push(record_checkpoint(subject.role, plan.kind, "01-help", sample,
                                    &tracker, recorder, sink));
                            }
                            Err(error) => {
                                interaction_ok = false;
                                checkpoints.push(failed_checkpoint("01-help", error));
                            }
                        }
                    }
                    let sent = send_planned_key(&mut *port, &mut tracker, plan, 2, "escape", &mut inputs);
                    interaction_ok &= sent;
                    if sent {
                        match wait_checkpoint(&mut *port, &mut tracker, plan, plan.kind, &subject.executable,
                            "02-root-restored", Instant::now() + CHECKPOINT_TIMEOUT) {
                            Ok((sample, _)) => {
                                restored_checkpoint_ok = true;
                                pre_ctrl_c_modes = Some(sample.modes.clone());
                                pre_ctrl_c_cursor = Some(sample.cursor.clone());
                                checkpoints.push(record_checkpoint(subject.role, plan.kind, "02-root-restored", sample,
                                    &tracker, recorder, sink));
                            }
                            Err(error) => {
                                interaction_ok = false;
                                checkpoints.push(failed_checkpoint("02-root-restored", error));
                            }
                        }
                    }
                }
                Err(error) => {
                    first_frame = Status::Fail;
                    interaction_ok = false;
                    checkpoints.push(failed_checkpoint("00-root", error));
                }
            }
        } else {
            checkpoints.push(failed_checkpoint("00-root", "application was not launched after build".into()));
            interaction_ok = false;
        }

        // Ctrl+C is the final real interaction, not a substitute for observed
        // exit status. Send it after the same root/help/Escape sequence on
        // successful runs; on earlier failure it remains a best-effort stop.
        let ctrl_c_sent = send_planned_key(&mut *port, &mut tracker, plan, 3, "ctrl-c", &mut inputs);
        if root_checkpoint_ok && help_checkpoint_ok && restored_checkpoint_ok {
            interaction_ok &= inputs.iter().all(|input| input.write_status == "written_and_recorded");
        }
        set_layer(
            &mut invocation,
            Layer::FirstFrame,
            first_frame,
            if first_frame == Status::Pass { "live PTY grid met the root readiness assertions" } else { "root readiness frame was not observed" },
            checkpoints.first().and_then(|cp| cp.live_text_sha256.clone()).into_iter().collect(),
        );

        let capture_ok = checkpoints.iter().all(|checkpoint| checkpoint.status == Status::Pass);
        let interaction_status = if interaction_ok && root_checkpoint_ok && help_checkpoint_ok && restored_checkpoint_ok && capture_ok {
            Status::Pass
        } else if first_frame == Status::Fail || checkpoints.iter().any(|checkpoint| checkpoint.status == Status::Fail) {
            Status::Fail
        } else {
            Status::Blocked
        };
        set_layer(&mut invocation, Layer::Interaction, interaction_status,
            "same F1, Escape, and Ctrl+C sequence and live help/root assertions are used for every role and invocation",
            inputs.iter().map(|input| format!("{}:{}:{}", input.sequence, input.key, input.write_status)).collect());

        let exit = match port.wait_exit(Instant::now() + EXIT_TIMEOUT) {
            Ok(exit) => {
                let status = if exit.success && exit.stream == "clean_eof" {
                    Status::Pass
                } else {
                    Status::Fail
                };
                set_layer(&mut invocation, Layer::Exit, status,
                    format!("direct child exit code {}; stream {}", exit.code, exit.stream),
                    vec![format!("signal_present={}", exit.signal_present), format!("ctrl_c_write={ctrl_c_sent}")]);
                Some(exit)
            }
            Err(error) => {
                set_layer(&mut invocation, Layer::Exit, Status::Blocked,
                    format!("child exit status unavailable: {error}"), vec![]);
                None
            }
        };

        let mut artifact_error = None;
        let final_sample = port.sample().ok();
        if let Some(sample) = &final_sample {
            if let Err(error) = tracker.update(plan.kind, &subject.executable, sample) {
                artifact_error = Some(format!("final PTY output capture incomplete: {error}"));
            }
            if sample.output_truncated || sample.output_bytes_total != sample.output_log.len() as u64 {
                artifact_error = Some("final PTY launch log was truncated or byte-inconsistent".into());
            }
            after_exit_modes = Some(sample.modes.clone());
            after_exit_cursor = Some(sample.cursor.clone());
        } else {
            artifact_error = Some("final PTY observation unavailable; launch log completeness unknown".into());
        }
        let full_log = final_sample.map(|sample| sample.output_log).unwrap_or_default();
        match sink.store(&format!("RUN-03/{}/{}/launch.raw", subject.role, invocation_slug(plan.kind)), &full_log) {
            Ok(artifact) => artifact_refs.push(artifact),
            Err(error) => artifact_error = Some(format!("launch log artifact write failed: {error}")),
        }
        if let Some(boundary) = tracker.running_boundary {
            if boundary <= full_log.len() {
                match sink.store(
                    &format!("RUN-03/{}/{}/application.raw", subject.role, invocation_slug(plan.kind)),
                    &full_log[boundary..],
                ) {
                    Ok(artifact) => artifact_refs.push(artifact),
                    Err(error) => artifact_error = Some(format!("application log artifact write failed: {error}")),
                }
            }
        }
        match serde_json::to_vec_pretty(&inputs)
            .map_err(|error| format!("serialize RUN-03 input events: {error}"))
            .and_then(|bytes| sink.store(&format!("RUN-03/{}/{}/inputs.json", subject.role, invocation_slug(plan.kind)), &bytes))
        {
            Ok(artifact) => artifact_refs.push(artifact),
            Err(error) => artifact_error = Some(format!("input event artifact write failed: {error}")),
        }
        let cleanup_report = port.close();
        match (&artifact_error, &cleanup_report.state) {
            (Some(error), _) => set_layer(&mut invocation, Layer::Cleanup, Status::Fail, error.clone(), vec![]),
            (None, CleanupState::Verified) => set_layer(&mut invocation, Layer::Cleanup, Status::Pass,
                "direct PTY teardown succeeded and verified child process group empty", vec![cleanup_report.direct_session_close.clone(), cleanup_report.process_group_containment.clone()]),
            (None, CleanupState::Unverified) => set_layer(&mut invocation, Layer::Cleanup, Status::Blocked,
                "PTY teardown ran but process-group containment could not be proven", vec![cleanup_report.direct_session_close.clone(), cleanup_report.process_group_containment.clone()]),
            (None, CleanupState::Failed) => set_layer(&mut invocation, Layer::Cleanup, Status::Fail,
                "PTY or child process-group cleanup failed", vec![cleanup_report.direct_session_close.clone(), cleanup_report.process_group_containment.clone()]),
        }

        set_layer(&mut invocation, Layer::Restoration, Status::Blocked,
            "required parent/slave termios and shell-continuation restoration are not exposed by this PTY adapter; recorded live child-PTY cursor/mode state is supporting evidence only",
            vec!["host_terminal_state=not_observed".into()]);
        set_layer(&mut invocation, Layer::Visual, Status::Blocked,
            "captures were produced, but no shared approved generation was admitted for visual comparison", vec![]);
        if subject.role == "reference" {
            set_layer(&mut invocation, Layer::Ownership, Status::NotApplicable,
                "reference implementation is not required to use candidate component ownership", vec![]);
        } else {
            set_layer(&mut invocation, Layer::Ownership, Status::Blocked,
                "command execution does not establish the independent candidate ownership source checks", vec![]);
        }

        RunEvidence {
            invocation,
            environment,
            pre_spawn_executable_sha256: Some(subject.executable_sha256.clone()),
            post_run_executable_sha256: None,
            checkpoints,
            inputs,
            exit,
            restoration: RestorationReceipt {
                status: Status::Blocked,
                reason: "child PTY mode state is recorded but parent/slave termios and shell continuation remain required and unobserved".into(),
                before_ctrl_c: pre_ctrl_c_modes,
                after_exit: after_exit_modes,
                before_ctrl_c_cursor: pre_ctrl_c_cursor,
                after_exit_cursor,
                host_terminal_state: "not_observed".into(),
                child_pty_state: "live cursor and DEC mode facts recorded from termpane observations".into(),
            },
            artifacts: artifact_refs,
            cleanup: cleanup_report,
        }
    }

    fn failed_checkpoint(id: &str, reason: String) -> CheckpointReceipt {
        CheckpointReceipt {
            id: id.to_string(),
            status: Status::Fail,
            reason,
            live_text_sha256: None,
            live_cursor: None,
            live_modes: None,
            capture_artifacts: Vec::new(),
            frame_digest: None,
        }
    }

    fn invocation_slug(kind: InvocationKind) -> &'static str {
        match kind {
            InvocationKind::ExactRootCommand => "cargo-run",
            InvocationKind::DirectExecutableDiagnostic => "direct-executable",
        }
    }

    fn set_layer(invocation: &mut InvocationReceipt, layer: Layer, status: Status, reason: impl Into<String>, evidence: Vec<String>) {
        invocation.layers.insert(layer, LayerReceipt { status, reason: reason.into(), evidence });
    }

    fn cleanup_not_run() -> CleanupRecord {
        CleanupRecord {
            state: CleanupState::Unverified,
            direct_session_close: "NOT_RUN".into(),
            process_group_containment: "NOT_RUN".into(),
            signalled_pids: Vec::new(),
            survivors: Vec::new(),
        }
    }

    fn hash_file(path: &Path) -> Result<String, String> {
        let mut file = File::open(path).map_err(|error| format!("open {} for hashing: {error}", path.display()))?;
        let mut hasher = Sha256::new();
        let mut buffer = [0; 64 * 1024];
        loop {
            let n = file.read(&mut buffer).map_err(|error| format!("hash {}: {error}", path.display()))?;
            if n == 0 { break; }
            hasher.update(&buffer[..n]);
        }
        Ok(format!("{:x}", hasher.finalize()))
    }

    fn sha256(bytes: &[u8]) -> String { format!("{:x}", Sha256::digest(bytes)) }

    fn valid_sha256(value: &str) -> bool {
        value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
    }

    pub fn execute_run03_pair(
        subjects: &[FrozenCommandSubject],
        environments: &BTreeMap<String, ExecutionEnvelope>,
        factory: &dyn PtyFactory,
        recorder: &mut dyn FrameRecorder,
        sink: &mut DirectoryArtifactSink,
    ) -> Result<(CommandReceipt, Vec<RunEvidence>), String> {
        if !matches!(std::env::consts::OS, "linux" | "macos") {
            return Err(format!("RUN-03 PTY execution is blocked on unverified platform {}", std::env::consts::OS));
        }
        let mut receipt = initial_receipt(subjects)?;
        let mut evidence = Vec::new();
        let protected_roots: Vec<PathBuf> = subjects.iter().map(|subject| subject.source_root.clone()).collect();
        let mut validated = BTreeMap::new();
        let mut preflight_failure = None;
        for subject in subjects {
            match environments.get(subject.role)
                .ok_or_else(|| format!("missing validated execution envelope for {}", subject.role))
                .and_then(|envelope| envelope.validate(subject, &protected_roots).map(|receipt| (envelope, receipt)))
                .and_then(|(envelope, receipt)| {
                    let envelope_root = fs::canonicalize(&envelope.artifact_root)
                        .map_err(|error| format!("canonicalize validated artifact root: {error}"))?;
                    if envelope_root != sink.root() {
                        return Err(format!("{} artifact sink root differs from its validated execution envelope", subject.role));
                    }
                    Ok((envelope, receipt))
                })
            {
                Ok((envelope, env_receipt)) => {
                    validated.insert(subject.role, (envelope, env_receipt));
                }
                Err(error) => preflight_failure = Some(error),
            }
        }
        if let Some(error) = preflight_failure {
            let run03 = receipt.rows.get_mut(2).expect("RUN-03 row exists");
            for invocation in run03.reference.iter_mut().chain(run03.candidate.iter_mut())
                .chain(run03.direct_executable_diagnostics.iter_mut())
            {
                for layer in [Layer::Build, Layer::Launch, Layer::FirstFrame, Layer::Interaction, Layer::Exit, Layer::Cleanup, Layer::Restoration, Layer::Visual, Layer::Ownership] {
                    set_layer(invocation, layer, Status::Blocked, format!("pair preflight blocked before any process launched: {error}"), vec![]);
                }
            }
            run03.status = Status::Blocked;
            return Ok((receipt, evidence));
        }
        for subject in subjects {
            let (envelope, env_receipt) = validated.get(subject.role)
                .ok_or_else(|| format!("validated environment missing for {}", subject.role))?;
            let plans = run03_plans(subject)?;
            for plan in &plans {
                let pre_spawn_hash = hash_file(&subject.executable).ok();
                let invocation = if pre_spawn_hash.as_deref() != Some(subject.executable_sha256.as_str()) {
                    let mut invocation = empty_invocation(subject, plan.kind, plan.argv.clone());
                    set_layer(&mut invocation, Layer::Launch, Status::Fail,
                        "frozen executable digest changed immediately before PTY spawn", vec![
                            format!("expected_sha256={}", subject.executable_sha256),
                            format!("actual_sha256={}", pre_spawn_hash.as_deref().unwrap_or("unavailable")),
                        ]);
                    for layer in [Layer::Build, Layer::FirstFrame, Layer::Interaction, Layer::Exit, Layer::Cleanup, Layer::Restoration, Layer::Visual, Layer::Ownership] {
                        let status = match (plan.kind, layer) {
                            (InvocationKind::DirectExecutableDiagnostic, Layer::Build) => Status::NotApplicable,
                            (InvocationKind::ExactRootCommand, Layer::Build) => Status::NotRun,
                            (_, Layer::Restoration | Layer::Visual | Layer::Ownership) => Status::Blocked,
                            _ => Status::NotRun,
                        };
                        set_layer(&mut invocation, layer, status, "invocation stopped before PTY spawn", vec![]);
                    }
                    RunEvidence {
                        invocation,
                        environment: Some(env_receipt.clone()),
                        pre_spawn_executable_sha256: pre_spawn_hash,
                        post_run_executable_sha256: None,
                        checkpoints: Vec::new(), inputs: Vec::new(), exit: None,
                        restoration: RestorationReceipt {
                            status: Status::Blocked,
                            reason: "invocation stopped before PTY spawn".into(),
                            before_ctrl_c: None, after_exit: None,
                            before_ctrl_c_cursor: None, after_exit_cursor: None,
                            host_terminal_state: "not_observed".into(), child_pty_state: "not_observed".into(),
                        }, artifacts: Vec::new(), cleanup: cleanup_not_run(),
                    }
                } else {
                    match factory.spawn(subject, plan, envelope) {
                    Ok(port) => {
                        let mut result = drive_invocation(subject, plan, Some(env_receipt.clone()), port, recorder, sink);
                        result.pre_spawn_executable_sha256 = pre_spawn_hash.clone();
                        let post_hash = hash_file(&subject.executable).ok();
                        if post_hash.as_deref() != Some(subject.executable_sha256.as_str()) {
                            let layer = if plan.kind == InvocationKind::ExactRootCommand { Layer::Build } else { Layer::Launch };
                            set_layer(&mut result.invocation, layer, Status::Fail,
                                "frozen Holla executable bytes changed during the invocation", vec![
                                    format!("expected_sha256={}", subject.executable_sha256),
                                    format!("actual_sha256={}", post_hash.as_deref().unwrap_or("unavailable")),
                                ]);
                        }
                        result.post_run_executable_sha256 = post_hash;
                        result
                    }
                    Err(error) => {
                        let mut invocation = empty_invocation(subject, plan.kind, plan.argv.clone());
                        set_layer(&mut invocation, Layer::Launch, Status::Fail, format!("PTY spawn failed: {error}"), vec![]);
                        if plan.kind == InvocationKind::DirectExecutableDiagnostic {
                            set_layer(&mut invocation, Layer::Build, Status::NotApplicable,
                                "separate direct-executable diagnostic does not invoke Cargo", vec![]);
                        }
                        for layer in [Layer::FirstFrame, Layer::Interaction, Layer::Exit, Layer::Cleanup, Layer::Restoration, Layer::Visual, Layer::Ownership] {
                            let status = if layer == Layer::Restoration || layer == Layer::Visual || layer == Layer::Ownership { Status::Blocked } else { Status::NotRun };
                            set_layer(&mut invocation, layer, status, "invocation did not reach this layer", vec![]);
                        }
                        RunEvidence {
                            invocation,
                            environment: Some(env_receipt.clone()),
                            pre_spawn_executable_sha256: pre_spawn_hash.clone(),
                            post_run_executable_sha256: None,
                            checkpoints: Vec::new(), inputs: Vec::new(), exit: None,
                            restoration: RestorationReceipt {
                                status: Status::Blocked,
                                reason: "PTY did not start; restoration was not observed".into(),
                                before_ctrl_c: None, after_exit: None,
                                before_ctrl_c_cursor: None, after_exit_cursor: None,
                                host_terminal_state: "not_observed".into(),
                                child_pty_state: "not_observed".into(),
                            }, artifacts: Vec::new(), cleanup: cleanup_not_run(),
                        }
                    }
                    }
                };
                let row = receipt.rows.get_mut(2).expect("RUN-03 row exists");
                let target = match (subject.role, plan.kind) {
                    ("reference", InvocationKind::ExactRootCommand) => row.reference.as_mut(),
                    ("candidate", InvocationKind::ExactRootCommand) => row.candidate.as_mut(),
                    ("reference", InvocationKind::DirectExecutableDiagnostic) => row.direct_executable_diagnostics.iter_mut().find(|item| item.role == "reference"),
                    ("candidate", InvocationKind::DirectExecutableDiagnostic) => row.direct_executable_diagnostics.iter_mut().find(|item| item.role == "candidate"),
                    _ => None,
                }.ok_or_else(|| "RUN-03 invocation slot missing".to_string())?;
                *target = invocation.invocation.clone();
                evidence.push(invocation);
            }
        }
        let run03 = receipt.rows.get_mut(2).expect("RUN-03 row exists");
        run03.status = aggregate_run03_status(run03);
        Ok((receipt, evidence))
    }

    fn aggregate_run03_status(row: &CommandRow) -> Status {
        let mut statuses = Vec::new();
        statuses.extend(row.reference.iter().flat_map(|item| item.layers.values().map(|layer| layer.status)));
        statuses.extend(row.candidate.iter().flat_map(|item| item.layers.values().map(|layer| layer.status)));
        statuses.extend(row.direct_executable_diagnostics.iter().flat_map(|item| item.layers.values().map(|layer| layer.status)));
        if statuses.iter().any(|status| *status == Status::Fail) { Status::Fail }
        else if statuses.iter().any(|status| *status == Status::Blocked || *status == Status::Unknown || *status == Status::NotRun) { Status::Blocked }
        else { Status::Pass }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::collections::VecDeque;
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;

        struct FakePty {
            samples: VecDeque<LiveSample>,
            current: LiveSample,
            inputs: Vec<Vec<u8>>,
            exit: Result<ExitRecord, String>,
            close_result: Result<(), String>,
            close_called: Arc<AtomicBool>,
            exited: bool,
        }

        impl FakePty {
            fn new(samples: Vec<LiveSample>, exit: Result<ExitRecord, String>, close_result: Result<(), String>) -> (Self, Arc<AtomicBool>) {
                // The real PTY output log is append-only; keep that property in
                // the fake while allowing each sample to expose a new grid.
                let mut accumulated = Vec::new();
                let mut cumulative = Vec::with_capacity(samples.len());
                for (index, mut sample) in samples.into_iter().enumerate() {
                    if index == 0 {
                        accumulated = sample.output_log.clone();
                    } else {
                        accumulated.extend_from_slice(format!("\n[fake observation {}]\n", sample.revision).as_bytes());
                        accumulated.extend_from_slice(sample.text.as_bytes());
                    }
                    sample.output_log = accumulated.clone();
                    sample.output_bytes_total = sample.output_log.len() as u64;
                    cumulative.push(sample);
                }
                let mut samples = VecDeque::from(cumulative);
                let current = samples.pop_front().expect("test sample");
                let close_called = Arc::new(AtomicBool::new(false));
                (Self { samples, current, inputs: Vec::new(), exit, close_result, close_called: close_called.clone(), exited: false }, close_called)
            }
        }

        impl CommandPty for FakePty {
            fn sample(&mut self) -> Result<LiveSample, String> { Ok(self.current.clone()) }
            fn wait_after(&mut self, _revision: u64, _deadline: Instant) -> Result<bool, String> {
                if let Some(next) = self.samples.pop_front() { self.current = next; Ok(true) } else { Ok(false) }
            }
            fn write_input(&mut self, bytes: &[u8]) -> Result<(), String> { self.inputs.push(bytes.to_vec()); Ok(()) }
            fn has_exited(&self) -> bool { self.exited }
            fn process_id(&self) -> Option<u32> { Some(42) }
            fn wait_exit(&mut self, _deadline: Instant) -> Result<ExitRecord, String> {
                match &self.exit { Ok(exit) => { self.exited = true; Ok(exit.clone()) }, Err(error) => Err(error.clone()) }
            }
            fn close(&mut self) -> CleanupRecord {
                self.close_called.store(true, Ordering::SeqCst);
                match &self.close_result {
                    Ok(()) => CleanupRecord {
                        state: CleanupState::Verified,
                        direct_session_close: "fake close joined".into(),
                        process_group_containment: "fake group empty".into(),
                        signalled_pids: Vec::new(),
                        survivors: Vec::new(),
                    },
                    Err(error) => CleanupRecord {
                        state: CleanupState::Failed,
                        direct_session_close: format!("fake close failed: {error}"),
                        process_group_containment: "fake group cleanup failed".into(),
                        signalled_pids: Vec::new(),
                        survivors: vec![42],
                    },
                }
            }
        }

        struct FakeRecorder;
        impl FrameRecorder for FakeRecorder {
            fn capture(&mut self, _role: &str, _kind: InvocationKind, checkpoint: &str,
                sample: &LiveSample, _recording: &tuiscotti::tui_shell::Recording,
                _sink: &mut dyn ArtifactSink) -> Result<CaptureProof, String> {
                Ok(CaptureProof {
                    artifacts: vec![], frame_digest: checkpoint.into(),
                    replay_text_matches_live: true, replay_cursor_matches_live: true,
                })
            }
        }

        #[derive(Default)]
        struct MemorySink;
        impl ArtifactSink for MemorySink {
            fn store(&mut self, relative_path: &str, bytes: &[u8]) -> Result<ArtifactRef, String> {
                Ok(ArtifactRef { path: relative_path.into(), sha256: sha256(bytes), bytes: bytes.len() as u64 })
            }
        }

        fn live_sample(revision: u64, text: &str) -> LiveSample {
            let log = format!("    Finished `release` profile [optimized] target(s) in 0.00s\n   Running `/frozen/ref/target/release/holla`\n{text}").into_bytes();
            LiveSample {
                revision, text: text.into(), output_bytes_total: log.len() as u64,
                output_truncated: false, output_log: log,
                cursor: CursorRecord { row: 0, col: 0, visible: true, style: 0, text_cursor_enable: None },
                modes: DisplayModesRecord {
                    alternate_screen: false, application_cursor: false, application_keypad: false,
                    bracketed_paste: false, focus_events: false, mouse_mode: "None".into(),
                    mouse_encoding: "None".into(), kitty_keyboard: 0, in_synchronized_update: false, mid_sequence: false,
                },
            }
        }

        fn frozen_subject() -> FrozenCommandSubject {
            FrozenCommandSubject {
                role: "reference",
                source_manifest: PathBuf::from("/frozen/ref/manifest.json"),
                source_manifest_sha256: "a".repeat(64),
                source_root: PathBuf::from("/frozen/ref/tree"),
                source_commit: "b".repeat(40),
                toolchain: "stable-x86_64-unknown-linux-gnu".into(),
                target_triple: "x86_64-unknown-linux-gnu".into(),
                executable: PathBuf::from("/frozen/ref/target/release/holla"),
                executable_sha256: "c".repeat(64),
            }
        }

        fn root_plan() -> InvocationPlan {
            run03_plans(&frozen_subject()).unwrap()[0].clone()
        }

        #[test]
        fn input_bytes_match_pinned_tuiscotti_key_encoder() {
            assert_eq!(INPUT_F1, b"\x1bOP");
            assert_eq!(INPUT_ESCAPE, b"\x1b");
            assert_eq!(INPUT_CTRL_C, b"\x03");
        }

        #[test]
        fn cargo_marker_requires_the_exact_executable_and_complete_line() {
            let expected = Path::new("/frozen/ref/target/release/holla");
            let finished = b"    Finished `release` profile [optimized] target(s) in 0.01s\n";
            let expected_running = b"   Running `/frozen/ref/target/release/holla`\n";
            let finish_boundary = cargo_finished_boundary(finished).unwrap().unwrap();
            assert_eq!(finish_boundary, finished.len());
            assert_eq!(cargo_finished_boundary(b"    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.01s\n").unwrap(), None);
            assert_eq!(cargo_finished_boundary(b"    Finished `release` profile").unwrap(), None);
            assert_eq!(cargo_running_boundary(b"\x1b[1mRunning `", expected, None).unwrap(), None);
            assert!(cargo_running_boundary(b"Running `/wrong/holla`\n", expected, Some(0)).is_err());
            let build_script_running = b"   Running `/frozen/ref/target/release/build/example/build-script-build`\n";
            let transcript = [build_script_running.as_slice(), finished.as_slice(),
                b"   Running `/frozen/ref/target/release/holla`\n"].concat();
            let ordered_finished = cargo_finished_boundary(&transcript).unwrap().unwrap();
            assert_eq!(cargo_running_boundary(&transcript, expected, Some(ordered_finished)).unwrap(), Some(transcript.len()));
            let transcript = [finished.as_slice(), b"\x1b[32mRunning\x1b[0m `/frozen/ref/target/release/holla`\n"].concat();
            assert_eq!(cargo_running_boundary(&transcript, expected, Some(finish_boundary)).unwrap(), Some(transcript.len()));
            assert!(cargo_running_boundary(b"Running `/frozen/ref/target/release/holla` --unexpected\n", expected, Some(0)).is_err());
            assert_eq!(cargo_running_boundary(b"notice: Running `/frozen/ref/target/release/holla`\n", expected, Some(0)).unwrap(), None);
            let early = [expected_running.as_slice(), finished.as_slice()].concat();
            let late_finished = cargo_finished_boundary(&early).unwrap().unwrap();
            assert!(cargo_running_boundary(&early, expected, Some(late_finished)).is_err());
        }

        #[test]
        fn cargo_config_audit_rejects_rustflags_in_all_supported_toml_spellings() {
            assert_eq!(first_forbidden_cargo_config_line("[build]\nrustflags = [\"-C\", \"opt-level=0\"]\n"), Some(2));
            assert_eq!(first_forbidden_cargo_config_line("[target.x86_64-unknown-linux-gnu]\nrustflags = [\"--cfg\", \"unsafe_override\"]\n"), Some(2));
            assert_eq!(first_forbidden_cargo_config_line("build.rustflags = [\"-C\", \"opt-level=0\"]\n"), Some(1));
            assert_eq!(first_forbidden_cargo_config_line("target.x86_64-unknown-linux-gnu.rustflags = [\"--cfg\", \"unsafe_override\"]\n"), Some(1));
            assert_eq!(first_forbidden_cargo_config_line("build = { rustflags = [\"-C\", \"opt-level=0\"] }\n"), Some(1));
            assert_eq!(first_forbidden_cargo_config_line("[build]\njobs = 2\n"), None);
        }

        #[test]
        fn invocation_plans_keep_exact_cargo_argv_and_direct_executable_separate() {
            let plans = run03_plans(&frozen_subject()).unwrap();
            assert_eq!(plans[0].argv, ["cargo", "run", "--release", "--bin", "holla"]);
            assert_eq!(plans[1].argv, ["/frozen/ref/target/release/holla"]);
            assert_eq!(plans[0].input, plans[1].input);
            assert_eq!(plans[0].size, (120, 40));
        }

        #[test]
        fn success_keeps_each_live_layer_separate_and_restoration_blocked() {
            let samples = vec![
                live_sample(1, "holla❯\nType Search"),
                live_sample(2, "Key reference\nEverywhere"),
                live_sample(3, "holla❯\nType Search"),
            ];
            let (port, closed) = FakePty::new(samples, Ok(ExitRecord { success: true, code: 0, signal_present: false, stream: "clean_eof".into() }), Ok(()));
            let plan = root_plan();
            let outcome = drive_invocation(&frozen_subject(), &plan, None, Box::new(port), &mut FakeRecorder, &mut MemorySink);
            assert_eq!(outcome.invocation.layers[&Layer::Build].status, Status::Pass);
            assert_eq!(outcome.invocation.layers[&Layer::Launch].status, Status::Pass);
            assert_eq!(outcome.invocation.layers[&Layer::FirstFrame].status, Status::Pass);
            assert_eq!(outcome.invocation.layers[&Layer::Interaction].status, Status::Pass);
            assert_eq!(outcome.invocation.layers[&Layer::Exit].status, Status::Pass);
            assert_eq!(outcome.invocation.layers[&Layer::Cleanup].status, Status::Pass);
            assert_eq!(outcome.invocation.layers[&Layer::Restoration].status, Status::Blocked);
            assert!(closed.load(Ordering::SeqCst));
        }

        #[test]
        fn failed_help_assertion_remains_visible_and_does_not_skip_cleanup() {
            let samples = vec![
                live_sample(1, "holla❯\nType Search"),
                live_sample(2, "help failed"),
                live_sample(3, "holla❯\nType Search"),
            ];
            let (port, closed) = FakePty::new(samples, Ok(ExitRecord { success: true, code: 0, signal_present: false, stream: "clean_eof".into() }), Ok(()));
            let outcome = drive_invocation(&frozen_subject(), &root_plan(), None, Box::new(port), &mut FakeRecorder, &mut MemorySink);
            assert_eq!(outcome.invocation.layers[&Layer::Interaction].status, Status::Fail);
            assert!(outcome.checkpoints.iter().any(|checkpoint| checkpoint.id == "01-help"
                && checkpoint.status == Status::Fail
                && checkpoint.reason == "checkpoint 01-help timed out"));
            assert_eq!(outcome.invocation.layers[&Layer::Cleanup].status, Status::Pass);
            assert!(closed.load(Ordering::SeqCst));
        }

        #[test]
        fn missing_exit_is_blocked_and_still_runs_owned_cleanup() {
            let samples = vec![
                live_sample(1, "holla❯\nType Search"),
                live_sample(2, "Key reference\nEverywhere"),
                live_sample(3, "holla❯\nType Search"),
            ];
            let (port, closed) = FakePty::new(samples, Err("timeout".into()), Ok(()));
            let outcome = drive_invocation(&frozen_subject(), &root_plan(), None, Box::new(port), &mut FakeRecorder, &mut MemorySink);
            assert_eq!(outcome.invocation.layers[&Layer::Exit].status, Status::Blocked);
            assert_eq!(outcome.invocation.layers[&Layer::Cleanup].status, Status::Pass);
            assert_eq!(outcome.exit, None);
            assert!(closed.load(Ordering::SeqCst));
        }

        #[test]
        fn cleanup_error_is_not_hidden_by_successful_exit() {
            let samples = vec![
                live_sample(1, "holla❯\nType Search"),
                live_sample(2, "Key reference\nEverywhere"),
                live_sample(3, "holla❯\nType Search"),
            ];
            let (port, closed) = FakePty::new(samples, Ok(ExitRecord { success: true, code: 0, signal_present: false, stream: "clean_eof".into() }), Err("join failed".into()));
            let outcome = drive_invocation(&frozen_subject(), &root_plan(), None, Box::new(port), &mut FakeRecorder, &mut MemorySink);
            assert_eq!(outcome.invocation.layers[&Layer::Exit].status, Status::Pass);
            assert_eq!(outcome.invocation.layers[&Layer::Cleanup].status, Status::Fail);
            assert!(closed.load(Ordering::SeqCst));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_registry_reserves_exactly_run01_through_run07() {
        assert_eq!(COMMANDS.len(), 7);
        assert_eq!(
            COMMANDS.iter().map(|row| (row.0, row.1)).collect::<Vec<_>>(),
            vec![
                ("RUN-01", "cargo run --release --bin showcase"),
                ("RUN-02", "cargo run --release --bin jackin-preview"),
                ("RUN-03", "cargo run --release --bin holla"),
                ("RUN-04", "cargo run --release --bin tablepro"),
                ("RUN-05", "cargo run --release --bin tablepro -- --connect Production"),
                ("RUN-06", "cargo run --release --bin jackin-preview -- --scenario accounts-mixed"),
                ("RUN-07", "cargo run --release --bin holla -- --scenario remote-host"),
            ]
        );
    }

    #[test]
    fn run03_root_argv_is_exact_and_has_no_scenario_or_pause_override() {
        let subject = FrozenCommandSubject {
            role: "reference",
            source_manifest: PathBuf::from("/frozen/reference/manifest.json"),
            source_manifest_sha256: "a".repeat(64),
            source_root: PathBuf::from("/frozen/reference/tree"),
            source_commit: "b".repeat(40),
            toolchain: "stable-x86_64-unknown-linux-gnu".into(),
            target_triple: "x86_64-unknown-linux-gnu".into(),
            executable: PathBuf::from("/frozen/reference/target/release/holla"),
            executable_sha256: "c".repeat(64),
        };
        let plans = run03_plans(&subject).unwrap();
        assert_eq!(plans[0].argv, ["cargo", "run", "--release", "--bin", "holla"]);
        assert_eq!(plans[0].cwd, subject.source_root);
        assert!(!plans[0].argv.iter().any(|arg| arg == "--scenario" || arg == "paused"));
        assert_eq!(plans[1].argv, ["/frozen/reference/target/release/holla"]);
        assert_eq!(plans[1].kind, InvocationKind::DirectExecutableDiagnostic);
    }

    #[test]
    fn all_run03_invocations_share_one_input_program_and_geometry() {
        let subject = FrozenCommandSubject {
            role: "candidate",
            source_manifest: PathBuf::from("/frozen/candidate/manifest.json"),
            source_manifest_sha256: "a".repeat(64),
            source_root: PathBuf::from("/frozen/candidate/tree"),
            source_commit: "b".repeat(40),
            toolchain: "stable-x86_64-unknown-linux-gnu".into(),
            target_triple: "x86_64-unknown-linux-gnu".into(),
            executable: PathBuf::from("/frozen/candidate/target/release/holla"),
            executable_sha256: "c".repeat(64),
        };
        let candidate = run03_plans(&subject).unwrap();
        let reference = run03_plans(&FrozenCommandSubject {
            role: "reference",
            source_manifest: PathBuf::from("/frozen/reference/manifest.json"),
            source_manifest_sha256: "d".repeat(64),
            source_root: PathBuf::from("/frozen/reference/tree"),
            source_commit: "e".repeat(40),
            toolchain: "stable-x86_64-unknown-linux-gnu".into(),
            target_triple: "x86_64-unknown-linux-gnu".into(),
            executable: PathBuf::from("/frozen/reference/target/release/holla"),
            executable_sha256: "f".repeat(64),
        })
        .unwrap();
        assert_eq!(candidate[0].input, reference[0].input);
        assert_eq!(candidate[0].input, candidate[1].input);
        assert_eq!(candidate[0].input, reference[1].input);
        assert_eq!(candidate[0].size, (120, 40));
        assert_eq!(candidate[0].size, candidate[1].size);
        assert_eq!(candidate[0].size, reference[0].size);
        assert_eq!(candidate[0].size, reference[1].size);
    }

    #[test]
    fn initial_receipt_keeps_every_command_not_run() {
        let receipt = initial_receipt(&[
            FrozenCommandSubject {
                role: "reference",
                source_manifest: PathBuf::from("/frozen/reference/manifest.json"),
                source_manifest_sha256: "a".repeat(64),
                source_root: PathBuf::from("/frozen/reference/tree"),
                source_commit: "b".repeat(40),
                toolchain: "stable-x86_64-unknown-linux-gnu".into(),
                target_triple: "x86_64-unknown-linux-gnu".into(),
                executable: PathBuf::from("/frozen/reference/target/release/holla"),
                executable_sha256: "c".repeat(64),
            },
            FrozenCommandSubject {
                role: "candidate",
                source_manifest: PathBuf::from("/frozen/candidate/manifest.json"),
                source_manifest_sha256: "d".repeat(64),
                source_root: PathBuf::from("/frozen/candidate/tree"),
                source_commit: "e".repeat(40),
                toolchain: "stable-x86_64-unknown-linux-gnu".into(),
                target_triple: "x86_64-unknown-linux-gnu".into(),
                executable: PathBuf::from("/frozen/candidate/target/release/holla"),
                executable_sha256: "f".repeat(64),
            },
        ]).unwrap();
        assert_eq!(receipt.rows.len(), 7);
        assert!(receipt.rows.iter().all(|row| row.status == Status::NotRun));
        assert!(receipt.rows.iter().all(|row| {
            row.reference.as_ref().unwrap().layers.values().all(|layer| layer.status == Status::NotRun)
                && row.candidate.as_ref().unwrap().layers.values().all(|layer| layer.status == Status::NotRun)
        }));
        assert_eq!(receipt.rows[2].direct_executable_diagnostics.len(), 2);
        assert!(receipt.rows[2].direct_executable_diagnostics.iter().all(|run| run.layers.values().all(|layer| layer.status == Status::NotRun)));
        assert!(receipt.rows.iter().enumerate().all(|(index, row)| index == 2 || row.direct_executable_diagnostics.is_empty()));
    }

    #[test]
    fn lane_declares_resource_limits_and_native_platform_boundary() {
        assert_eq!(MAX_CONCURRENT_PTYS, 2);
        assert_eq!(CARGO_BUILD_JOBS, 2);
        assert_eq!(platform_eligibility("linux"), PlatformEligibility::SupportedLinuxOrMacos);
        assert_eq!(platform_eligibility("macos"), PlatformEligibility::SupportedLinuxOrMacos);
        assert_eq!(platform_eligibility("windows"), PlatformEligibility::BlockedUnsupportedOrUnverified);
    }
}

}
