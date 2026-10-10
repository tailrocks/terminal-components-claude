mod command {
//! Native command selectors and the shared RUN-01 through RUN-07 executor for this
//! `termrock-e2e` integration-test module, adapted
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
            reason: "no native command invocation has executed".into(),
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
    pub command_id: &'static str,
    pub kind: InvocationKind,
    /// Exact argv after PATH resolution. The root-command argv stays the
    /// transcript command; the executable diagnostic has its own argv.
    pub argv: Vec<String>,
    /// Resolved from the frozen source manifest by the caller, never from a
    /// current checkout, branch name, or test working directory.
    pub cwd: PathBuf,
    pub size: (u16, u16),
    pub input: Vec<InputEvent>,
    /// Same role-independent app-level values for both subjects. The exact
    /// Cargo command remains in `argv` and is never rewritten by these values.
    pub case_environment: BTreeMap<String, String>,
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
        /// Earlier checkpoints whose positive predicates must pass before
        /// this checkpoint's negative predicates are meaningful.
        requires: &'static [&'static str],
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
    /// Invocation-specific direct binary runs are separate diagnostics and
    /// never overwrite the exact Cargo-command result.
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

/// A command selector carries exact Cargo argv and a separate direct-binary
/// diagnostic argv. Program and executor readiness are tracked separately.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub enum CommandExecutionReadiness {
    SupportedByCurrentExecutor,
    Blocked { reason: String },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub enum CommandProgramReadiness {
    Ready,
    Blocked { reason: String },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NativeCommandSelection {
    pub id: &'static str,
    pub exact_command: &'static str,
    pub binary_name: &'static str,
    pub cargo_argv: Vec<String>,
    pub app_args: Vec<String>,
    pub direct_argv: Vec<String>,
    pub source_citations: Vec<&'static str>,
    pub program_readiness: CommandProgramReadiness,
    pub execution_readiness: CommandExecutionReadiness,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CommandProgram {
    pub command_id: &'static str,
    pub input: &'static [InputEvent],
    pub case_environment: &'static [(&'static str, &'static str)],
    pub source_citations: &'static [&'static str],
}

/// These immutable input/assertion programs are shared by reference,
/// candidate, and the separate direct-executable diagnostic. Each command
/// remains byte-for-byte the exact root invocation recorded in COMMANDS.
pub const RUN01_INPUT: &[InputEvent] = &[
    InputEvent::WaitFor {
        checkpoint: "00-overview",
        contains: &["Junie", "Design system", "/ Foundations / Overview"],
        absent: &[],
        requires: &[],
    },
    InputEvent::Press("]"),
    InputEvent::WaitFor {
        checkpoint: "01-buttons",
        contains: &["/ Components / Buttons"],
        absent: &[],
        requires: &["00-overview"],
    },
    InputEvent::Press("q"),
];

pub const RUN02_INPUT: &[InputEvent] = &[
    InputEvent::WaitFor {
        checkpoint: "00-intro",
        contains: &["Stand up, operator…"],
        absent: &[],
        requires: &[],
    },
    InputEvent::WaitFor {
        checkpoint: "01-manager",
        contains: &["Workspaces", "inside the Construct"],
        absent: &[],
        requires: &["00-intro"],
    },
    InputEvent::Press("down"),
    InputEvent::WaitFor {
        checkpoint: "02-new-workspace",
        contains: &["Enter starts the five-step create chain"],
        absent: &[],
        requires: &["01-manager"],
    },
    InputEvent::Press("q"),
];

pub const RUN03_INPUT: &[InputEvent] = &[
    InputEvent::WaitFor {
        checkpoint: "00-root",
        contains: &["holla❯", "Type Search"],
        absent: &[],
        requires: &[],
    },
    InputEvent::Press("f1"),
    InputEvent::WaitFor {
        checkpoint: "01-help",
        contains: &["Key reference", "Everywhere"],
        absent: &[],
        requires: &["00-root"],
    },
    InputEvent::Press("escape"),
    InputEvent::WaitFor {
        checkpoint: "02-root-restored",
        contains: &["holla❯", "Type Search"],
        absent: &["Key reference"],
        requires: &["01-help"],
    },
    // Holla source maps Ctrl+C to immediate quit. The event is a request; it
    // cannot itself prove the child exited or the parent terminal was restored.
    InputEvent::Press("ctrl-c"),
];

pub const RUN04_INPUT: &[InputEvent] = &[
    InputEvent::WaitFor {
        checkpoint: "00-connections",
        contains: &["Connections", "Local PostgreSQL", "Production"],
        absent: &[],
        requires: &[],
    },
    InputEvent::Press("ctrl-n"),
    InputEvent::WaitFor {
        checkpoint: "01-new-connection-form",
        contains: &["Name", "Host", "Save"],
        absent: &[],
        requires: &["00-connections"],
    },
    InputEvent::Press("escape"),
    InputEvent::WaitFor {
        checkpoint: "02-connections-restored",
        contains: &["Connections", "Local PostgreSQL"],
        absent: &["Save"],
        requires: &["01-new-connection-form"],
    },
    InputEvent::Press("ctrl-c"),
];

pub const RUN05_INPUT: &[InputEvent] = &[
    InputEvent::WaitFor {
        checkpoint: "00-explorer",
        contains: &["Explorer", "Query 1"],
        absent: &[],
        requires: &[],
    },
    InputEvent::Press("down"),
    InputEvent::Press("down"),
    InputEvent::Press("down"),
    InputEvent::Press("down"),
    InputEvent::Press("down"),
    InputEvent::Press("enter"),
    InputEvent::WaitFor {
        checkpoint: "01-order-detail",
        contains: &["public › orders", "10000"],
        absent: &[],
        requires: &["00-explorer"],
    },
    InputEvent::Press("ctrl-d"),
    InputEvent::WaitFor {
        checkpoint: "02-foreign-keys",
        contains: &["Foreign keys"],
        absent: &["10000"],
        requires: &["01-order-detail"],
    },
    InputEvent::Press("ctrl-c"),
];

pub const RUN06_INPUT: &[InputEvent] = &[
    InputEvent::WaitFor {
        checkpoint: "00-accounts",
        contains: &["Overview", "Work"],
        absent: &[],
        requires: &[],
    },
    InputEvent::Press("down"),
    InputEvent::Press("down"),
    InputEvent::Press("down"),
    InputEvent::Press("enter"),
    InputEvent::WaitFor {
        checkpoint: "01-work-account-drawer",
        contains: &["Team"],
        absent: &[],
        requires: &["00-accounts"],
    },
    InputEvent::Press("escape"),
    InputEvent::Press("escape"),
    InputEvent::WaitFor {
        checkpoint: "02-manager-restored",
        contains: &["jackin❯", "Current directory"],
        absent: &["Team"],
        requires: &["01-work-account-drawer"],
    },
    InputEvent::Press("q"),
];

pub const RUN07_INPUT: &[InputEvent] = &[
    InputEvent::WaitFor {
        checkpoint: "00-remote-host",
        contains: &["◆ prod-eu-1 · production", "on prod-eu-1"],
        absent: &[],
        requires: &[],
    },
    InputEvent::Press("f1"),
    InputEvent::WaitFor {
        checkpoint: "01-help",
        contains: &["Key reference"],
        absent: &[],
        requires: &["00-remote-host"],
    },
    InputEvent::Press("escape"),
    InputEvent::WaitFor {
        checkpoint: "02-remote-host-restored",
        contains: &["◆ prod-eu-1 · production", "on prod-eu-1"],
        absent: &["Key reference"],
        requires: &["01-help"],
    },
    InputEvent::Press("ctrl-c"),
];

pub const COMMAND_PROGRAMS: [CommandProgram; 7] = [
    CommandProgram {
        command_id: "RUN-01",
        input: RUN01_INPUT,
        case_environment: &[],
        source_citations: &[
            "candidate:crates/showcase-ui/src/app.rs:456-482,815-854,1282-1295,1375-1378,1459-1486",
            "reference:src/bin/showcase/app.rs:72-81,190-193,261-323,591-607,874-880,963-975; src/bin/showcase/main.rs:114-116",
            "pack:COMMANDS.md:10-16",
        ],
    },
    CommandProgram {
        command_id: "RUN-02",
        input: RUN02_INPUT,
        case_environment: &[],
        source_citations: &[
            "candidate:crates/jackin-preview-presentation/src/rain.rs:321-327",
            "candidate:crates/jackin-preview-app/src/cli.rs:55-57; crates/jackin-preview-app/src/app.rs:900-906,8836-8843",
            "candidate:crates/jackin-preview-app/src/app.rs:6782 (outside-the-Construct text)",
            "candidate:crates/jackin-preview-host-ui/src/manager.rs:113-122,337-353,453-463,1196-1199",
            "reference:src/bin/jackin_preview/rain.rs:475-481",
            "reference:src/bin/jackin_preview/app.rs:217-225,467-471,660-666,836-863,2170-2173; src/bin/jackin_preview/screens/manager.rs:104-108,142-150,278-282,2641-2644",
            "reference:src/bin/jackin_preview/app.rs:2428 (outside-the-Construct text)",
            "pack:COMMANDS.md:10-16",
        ],
    },
    CommandProgram {
        command_id: "RUN-03",
        input: RUN03_INPUT,
        case_environment: &[("HOLLA_NO_HISTORY", "1")],
        source_citations: &[
            "reference:tests/harness/tests/visual_baseline/holla_pending_h6.rs:298-358",
            "candidate:crates/termrock-conformance/tests/visual_baseline/holla_pending_h6.rs:309-369",
            "shared:cases/registry.json:1-55 (HELP-HOLLA-004 case environment)",
            "pack:COMMANDS.md:10-16",
        ],
    },
    CommandProgram {
        command_id: "RUN-04",
        input: RUN04_INPUT,
        case_environment: &[],
        source_citations: &[
            "candidate:crates/tablepro-ui/src/app.rs:516-540,764-802,7654-7668,1546-1580; crates/tablepro-ui/src/connections.rs:90-115",
            "reference:src/bin/tablepro/app.rs:145-173,250-276,501-511; src/bin/tablepro/connections.rs:139-210",
            "pack:COMMANDS.md:10-16",
        ],
    },
    CommandProgram {
        command_id: "RUN-05",
        input: RUN05_INPUT,
        case_environment: &[],
        source_citations: &[
            "candidate:crates/termrock-conformance/tests/visual_baseline/tablepro_journeys.rs:222-265",
            "candidate:crates/tablepro-ui/src/app.rs:7654-7668,1546-1580",
            "reference:src/bin/tablepro/app.rs:250-276; src/bin/tablepro/app_tests.rs:178-190,259-277; src/bin/tablepro/db.rs:941",
            "candidate:crates/tablepro-demo/src/db.rs:1",
            "pack:COMMANDS.md:10-16",
        ],
    },
    CommandProgram {
        command_id: "RUN-06",
        input: RUN06_INPUT,
        case_environment: &[],
        source_citations: &[
            "candidate:crates/jackin-preview-app/src/cli.rs:50-90; crates/jackin-preview-app/src/app.rs:900-907,5391-5419,8717-8755",
            "candidate:crates/jackin-preview-host-ui/src/accounts.rs:78-83,112-119,127-167,783-787,1931-1938",
            "candidate:crates/jackin-preview-host-ui/src/manager.rs:345 (Current directory label)",
            "candidate:crates/jackin-preview-sim/src/fixtures/pinned.rs:302-360",
            "candidate:crates/jackin-preview-app/src/app.rs:8836-8843",
            "reference:src/bin/jackin_preview/app.rs:246-248,660-666,2170-2173; src/bin/jackin_preview/screens/accounts.rs:122-126,1459-1464,2090-2104,2111-2119,2142-2145",
            "reference:src/bin/jackin_preview/screens/manager.rs:150 (Current directory label)",
            "reference:src/bin/jackin_preview/domain/fixtures.rs:697-759",
            "pack:COMMANDS.md:10-16",
        ],
    },
    CommandProgram {
        command_id: "RUN-07",
        input: RUN07_INPUT,
        case_environment: &[],
        source_citations: &[
            "candidate:crates/holla-domain/src/scenario.rs:23-29,140-152; crates/holla-sim/src/fixtures.rs:2474-2492",
            "reference:src/bin/holla/scenario.rs:23-29,140-152; src/domain/fixtures.rs:2474-2492",
            "reference:src/bin/holla/app.rs:545-565,768-879",
            "pack:COMMANDS.md:10-16",
        ],
    },
];

pub fn command_program(command_id: &str) -> Option<&'static CommandProgram> {
    COMMAND_PROGRAMS.iter().find(|program| program.command_id == command_id)
}

pub fn validate_command_program(program: &CommandProgram) -> Result<(), String> {
    if program.source_citations.is_empty() || program.source_citations.iter().any(|item| item.trim().is_empty()) {
        return Err(format!("{} program needs source citations", program.command_id));
    }
    if program.input.is_empty() {
        return Err(format!("{} program has no input steps", program.command_id));
    }
    let mut prior_checkpoints = Vec::new();
    let mut press_count = 0;
    for (index, event) in program.input.iter().enumerate() {
        match event {
            InputEvent::WaitFor { checkpoint, contains, absent, requires } => {
                if checkpoint.trim().is_empty() || contains.is_empty() || contains.iter().any(|needle| needle.trim().is_empty()) {
                    return Err(format!("{} checkpoint {checkpoint:?} needs positive readiness text", program.command_id));
                }
                if prior_checkpoints.contains(checkpoint) {
                    return Err(format!("{} checkpoint {checkpoint:?} is duplicated", program.command_id));
                }
                if index > 0 && requires.is_empty() {
                    return Err(format!("{} checkpoint {checkpoint:?} needs an earlier positive prerequisite", program.command_id));
                }
                if !absent.is_empty() && requires.is_empty() {
                    return Err(format!("{} negative checkpoint {checkpoint:?} has no positive prerequisite", program.command_id));
                }
                for required in *requires {
                    if !prior_checkpoints.contains(required) {
                        return Err(format!("{} checkpoint {checkpoint:?} requires non-prior checkpoint {required:?}", program.command_id));
                    }
                }
                prior_checkpoints.push(*checkpoint);
            }
            InputEvent::Press(key) => {
                if key.trim().is_empty() {
                    return Err(format!("{} has an empty key input", program.command_id));
                }
                press_count += 1;
            }
        }
    }
    if press_count == 0 || !matches!(program.input.last(), Some(InputEvent::Press("q" | "ctrl-c"))) {
        return Err(format!("{} program must end with a documented q or Ctrl+C exit input", program.command_id));
    }
    for (name, value) in program.case_environment {
        if name.trim().is_empty() || value.is_empty() {
            return Err(format!("{} has an empty case environment entry", program.command_id));
        }
    }
    Ok(())
}

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
            Ok(CommandRow {
                id,
                exact_command: command,
                status: Status::NotRun,
                reference: reference.map(invocation),
                candidate: candidate.map(invocation),
                direct_executable_diagnostics: Vec::new(),
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
    command_plans("RUN-03", subject)
}

/// Select one exact root command without translating its argv. The returned
/// direct argv is a separate diagnostic and never replaces the Cargo result.
/// Every command has one source-backed shared input program.
pub fn native_command_selection(
    command_id: &str,
    subject: &FrozenCommandSubject,
) -> Result<NativeCommandSelection, String> {
    if subject.role != "reference" && subject.role != "candidate" {
        return Err(format!("unsupported subject role {:?}", subject.role));
    }
    if !subject.source_manifest.is_absolute()
        || !subject.source_root.is_absolute()
        || !subject.executable.is_absolute()
    {
        return Err("frozen manifest, source root, and executable paths must be absolute".into());
    }
    let entry = COMMANDS
        .iter()
        .find(|(id, _, _)| *id == command_id)
        .ok_or_else(|| format!("unknown native command ID {command_id:?}"))?;
    let id = entry.0;
    let exact_command = entry.1;
    let argv = entry.2;
    let binary_index = argv
        .iter()
        .position(|argument| *argument == "--bin")
        .ok_or_else(|| format!("{id} command has no Cargo --bin target"))?;
    let binary_name = argv
        .get(binary_index + 1)
        .copied()
        .ok_or_else(|| format!("{id} Cargo --bin has no target name"))?;
    let separator = argv.iter().position(|argument| *argument == "--");
    let app_args: Vec<String> = separator
        .map(|position| argv[position + 1..].iter().map(|part| (*part).to_owned()).collect())
        .unwrap_or_default();
    let executable_name = subject
        .executable
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("{} executable path has no UTF-8 file name", subject.role))?;
    if executable_name != binary_name {
        return Err(format!(
            "{id} selects binary {binary_name:?}, but frozen {} executable is {executable_name:?}",
            subject.role
        ));
    }
    let executable = subject
        .executable
        .to_str()
        .ok_or_else(|| "direct executable path is not UTF-8".to_string())?;
    let mut direct_argv = vec![executable.to_owned()];
    direct_argv.extend(app_args.iter().cloned());
    let program = command_program(id);
    let program_readiness = if program.is_some() {
        CommandProgramReadiness::Ready
    } else {
        CommandProgramReadiness::Blocked {
            reason: format!("{id} has no source-reviewed shared interaction program"),
        }
    };
    let execution_readiness = if program_readiness == CommandProgramReadiness::Ready
        && platform_eligibility(std::env::consts::OS)
            == PlatformEligibility::SupportedLinuxOrMacos
    {
        CommandExecutionReadiness::SupportedByCurrentExecutor
    } else {
        CommandExecutionReadiness::Blocked {
            reason: format!("{id} has no runnable native PTY executor on this platform"),
        }
    };
    let source_citations = program
        .map(|program| program.source_citations.to_vec())
        .unwrap_or_default();
    Ok(NativeCommandSelection {
        id,
        exact_command,
        binary_name,
        cargo_argv: argv.iter().map(|part| (*part).to_owned()).collect(),
        app_args,
        direct_argv,
        source_citations,
        program_readiness,
        execution_readiness,
    })
}

/// Build paired plans from a command's reviewed shared program and controlled
/// case environment. Both subjects receive identical input bytes.
pub fn command_plans(
    command_id: &str,
    subject: &FrozenCommandSubject,
) -> Result<[InvocationPlan; 2], String> {
    let selection = native_command_selection(command_id, subject)?;
    if let CommandProgramReadiness::Blocked { reason } = selection.program_readiness {
        return Err(reason);
    }
    let program = command_program(command_id)
        .ok_or_else(|| format!("{command_id} has no shared command program"))?;
    validate_command_program(program)?;
    let root = subject.source_root.clone();
    let input = program.input.to_vec();
    let case_environment: BTreeMap<String, String> = program
        .case_environment
        .iter()
        .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
        .collect();
    let root_command = InvocationPlan {
        command_id: selection.id,
        kind: InvocationKind::ExactRootCommand,
        argv: selection.cargo_argv,
        cwd: root.clone(),
        size: (120, 40),
        input: input.clone(),
        case_environment: case_environment.clone(),
        max_concurrent_ptys: MAX_CONCURRENT_PTYS,
        cargo_build_jobs: CARGO_BUILD_JOBS,
        platform: platform_eligibility(std::env::consts::OS),
    };
    let direct_executable = InvocationPlan {
        command_id: selection.id,
        kind: InvocationKind::DirectExecutableDiagnostic,
        argv: selection.direct_argv,
        cwd: root,
        size: (120, 40),
        input,
        case_environment,
        max_concurrent_ptys: MAX_CONCURRENT_PTYS,
        cargo_build_jobs: CARGO_BUILD_JOBS,
        platform: platform_eligibility(std::env::consts::OS),
    };
    Ok([root_command, direct_executable])
}

pub fn command_row_index(command_id: &str) -> Result<usize, String> {
    COMMANDS
        .iter()
        .position(|(id, _, _)| *id == command_id)
        .ok_or_else(|| format!("unknown native command ID {command_id:?}"))
}

/// Initialize receipt slots for one selected command. Unselected rows stay
/// NOT_RUN, and the selected row gets only its own binary's direct diagnostic.
pub fn initial_command_receipt(
    subjects: &[FrozenCommandSubject],
    command_id: &str,
) -> Result<CommandReceipt, String> {
    let mut receipt = initial_receipt(subjects)?;
    let index = command_row_index(command_id)?;
    let mut direct = Vec::with_capacity(2);
    for role in ["reference", "candidate"] {
        let subject = subjects
            .iter()
            .find(|subject| subject.role == role)
            .ok_or_else(|| format!("receipt is missing {role} subject"))?;
        let selection = native_command_selection(command_id, subject)?;
        direct.push(empty_invocation(
            subject,
            InvocationKind::DirectExecutableDiagnostic,
            selection.direct_argv,
        ));
    }
    for row in &mut receipt.rows {
        row.direct_executable_diagnostics.clear();
    }
    receipt.rows[index].direct_executable_diagnostics = direct;
    Ok(receipt)
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

    const OUTPUT_CAP: usize = 64 * 1024 * 1024;
    const BUILD_TIMEOUT: Duration = Duration::from_secs(15 * 60);
    const CHECKPOINT_TIMEOUT: Duration = Duration::from_secs(20);
    const EXIT_TIMEOUT: Duration = Duration::from_secs(15);
    const INPUT_F1: &[u8] = b"\x1bOP";
    const INPUT_ESCAPE: &[u8] = b"\x1b";
    const INPUT_CTRL_C: &[u8] = b"\x03";
    const INPUT_CTRL_N: &[u8] = b"\x0e";
    const INPUT_DOWN: &[u8] = b"\x1b[B";
    const INPUT_ENTER: &[u8] = b"\r";
    const INPUT_CTRL_D: &[u8] = b"\x04";
    const INPUT_NEXT_PAGE: &[u8] = b"]";
    const INPUT_QUIT: &[u8] = b"q";

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

    #[derive(Clone, Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct ExecutionEnvelope {
        /// Child variables only. `SpawnParams::env_clear` ensures nothing
        /// else is inherited; the backend's required `SHELL` is overridden.
        pub variables: BTreeMap<String, String>,
        /// The exact files resolved through `PATH`; both hashes are recorded.
        pub cargo_path: PathBuf,
        pub cargo_sha256: String,
        pub rustc_path: PathBuf,
        pub rustc_sha256: String,
    /// Where the exact `cargo run` writes `target/release/<selected-bin>`.
        pub target_dir: PathBuf,
        pub artifact_root: PathBuf,
        /// Must equal two; the team Cargo slot is externally reserved.
        pub cargo_jobs: u8,
    }

    #[derive(Clone, Debug, Eq, PartialEq, Serialize)]
    pub struct EnvironmentReceipt {
        pub variable_hashes: BTreeMap<String, String>,
        pub case_environment_hashes: BTreeMap<String, String>,
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
        fn validate(
            &self,
            subject: &FrozenCommandSubject,
            target_name: &str,
            case_environment: &BTreeMap<String, String>,
            protected_roots: &[PathBuf],
        ) -> Result<EnvironmentReceipt, String> {
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
                return Err("frozen selected executable changed after source-pair admission".into());
            }
            if self.cargo_jobs != 2 || self.variables.get("CARGO_BUILD_JOBS").map(String::as_str) != Some("2") {
                return Err("environment must fix CARGO_BUILD_JOBS=2".into());
            }
            let actual_target = absolute_normalized(&self.target_dir)?;
            let expected_target = absolute_normalized(&subject.executable)
                .and_then(|path| path.parent().and_then(Path::parent).map(Path::to_path_buf)
                    .ok_or_else(|| "selected executable must be under target/release".to_string()))?;
            if actual_target != expected_target
                || subject.executable.file_name() != Some(OsStr::new(target_name))
                || subject.executable.parent().and_then(Path::file_name) != Some(OsStr::new("release"))
            {
                return Err(format!(
                    "CARGO_TARGET_DIR must resolve to the frozen target/release/{target_name} executable"
                ));
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
                    return Err(format!("environment key {name:?} is outside the controlled command allowlist"));
                }
            }
            for name in FORBIDDEN_ENV {
                if self.variables.contains_key(*name) {
                    return Err(format!("environment override {name} is forbidden for exact command execution"));
                }
            }
            for (name, value) in case_environment {
                if name.trim().is_empty()
                    || value.is_empty()
                    || REQUIRED_ENV.contains(&name.as_str())
                    || FORBIDDEN_ENV.contains(&name.as_str())
                {
                    return Err(format!("case environment input {name:?} is empty, controlled, or forbidden"));
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
            let case_environment_hashes = case_environment
                .iter()
                .map(|(name, value)| (name.clone(), sha256(value.as_bytes())))
                .collect();
            Ok(EnvironmentReceipt {
                variable_hashes,
                case_environment_hashes,
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
            command_id: &str,
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
                .current_dir(&plan.cwd);
            for (name, value) in &env.variables {
                params = params.env(name.as_str(), value.as_str());
            }
            for (name, value) in &plan.case_environment {
                params = params.env(name.as_str(), value.as_str());
            }
            let session = PtySession::spawn(
                &params,
                SessionOptions {
                    cols: plan.size.0,
                    rows: plan.size.1,
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
            command_id: &str,
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
                .map_err(|error| format!("replay {command_id} PTY output: {error}"))?;
            let frame = tuiscotti::render::frame_from_screen(&replay.screen, "default");
            let replay_text = frame.text();
            let text_matches = replay_text == sample.text;
            let cursor_matches = frame.cursor.x == sample.cursor.col
                && frame.cursor.y == sample.cursor.row
                && frame.cursor.visible == sample.cursor.visible;
            let profile = RenderProfile::vendored();
            let mut renderer = Renderer::for_render_profile(&profile)
                .map_err(|error| format!("initialize strict Tuiscotti renderer: {error}"))?;
            let bundle = capture_all(&mut renderer, &frame, &format!("{command_id}:{checkpoint}"))
                .map_err(|error| format!("capture {command_id} checkpoint: {error}"))?;
            let prefix = format!("{command_id}/{role}/{}/{}", invocation_slug(kind), checkpoint);
            let frame_digest = format!("{:016x}", frame.digest());
            let meta = serde_json::to_vec_pretty(&serde_json::json!({
                "schema": "termrock-spec/native-command-live-frame-capture-v1",
                "command_id": command_id,
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
            .map_err(|error| format!("serialize {command_id} frame metadata: {error}"))?;
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
    pub before_exit: Option<DisplayModesRecord>,
    pub after_exit: Option<DisplayModesRecord>,
    pub before_exit_cursor: Option<CursorRecord>,
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
                        return Err("expected executable Running marker appeared before the release Finished boundary".into());
                    }
                    // Cargo runs build scripts before its final Finished line.
                    // Their child commands also produce Running lines; ignore
                    // those pre-build-completion lines, while rejecting an
                    // early line that claims this exact selected executable.
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
            InputEvent::WaitFor { checkpoint, contains, absent, .. } if *checkpoint == id => {
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
        command_id: &str,
        role: &str,
        kind: InvocationKind,
        id: &str,
        sample: LiveSample,
        tracker: &OutputTracker,
        recorder: &mut dyn FrameRecorder,
        sink: &mut dyn ArtifactSink,
    ) -> CheckpointReceipt {
        let capture = recorder.capture(command_id, role, kind, id, &sample, &tracker.recording, sink);
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
            "ctrl-n" => Some(INPUT_CTRL_N),
            "down" => Some(INPUT_DOWN),
            "enter" => Some(INPUT_ENTER),
            "ctrl-d" => Some(INPUT_CTRL_D),
            "]" => Some(INPUT_NEXT_PAGE),
            "q" => Some(INPUT_QUIT),
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
        let mut passed_checkpoints = BTreeSet::new();
        let mut interaction_failed = false;
        let mut first_frame = None;
        let mut before_exit_modes = None;
        let mut after_exit_modes = None;
        let mut before_exit_cursor = None;
        let mut after_exit_cursor = None;
        let mut last_observation = None;
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
                            &format!("{}/{}/{}/build.raw", plan.command_id, subject.role, invocation_slug(plan.kind)),
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
                                "Cargo emitted a complete Running line naming the frozen selected release executable",
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
                                "Cargo exited before a verified Running line for the frozen selected executable", vec![]);
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

        if build_ready && started_pid.is_some() {
            let mut sequence = 0u8;
            let last_index = plan.input.len().saturating_sub(1);
            for (index, event) in plan.input.iter().enumerate() {
                match event {
                    InputEvent::WaitFor { checkpoint, requires, .. } => {
                        let missing = requires.iter().find(|required| !passed_checkpoints.contains(**required));
                        if let Some(required) = missing {
                            interaction_failed = true;
                            checkpoints.push(blocked_checkpoint(
                                checkpoint,
                                format!("checkpoint {checkpoint} requires earlier positive checkpoint {required}"),
                            ));
                            continue;
                        }
                        match wait_checkpoint(
                            &mut *port,
                            &mut tracker,
                            plan,
                            plan.kind,
                            &subject.executable,
                            checkpoint,
                            Instant::now() + CHECKPOINT_TIMEOUT,
                        ) {
                            Ok((sample, _)) => {
                                if first_frame.is_none() {
                                    first_frame = Some(Status::Pass);
                                }
                                before_exit_modes = Some(sample.modes.clone());
                                before_exit_cursor = Some(sample.cursor.clone());
                                last_observation = Some(sample.clone());
                                let receipt = record_checkpoint(
                                    plan.command_id,
                                    subject.role,
                                    plan.kind,
                                    checkpoint,
                                    sample,
                                    &tracker,
                                    recorder,
                                    sink,
                                );
                                if receipt.status == Status::Pass {
                                    passed_checkpoints.insert(*checkpoint);
                                } else {
                                    interaction_failed = true;
                                }
                                checkpoints.push(receipt);
                            }
                            Err(error) => {
                                if first_frame.is_none() {
                                    first_frame = Some(Status::Fail);
                                }
                                interaction_failed = true;
                                checkpoints.push(failed_checkpoint(checkpoint, error));
                            }
                        }
                    }
                    InputEvent::Press(key) => {
                        sequence = sequence.saturating_add(1);
                        let is_final_exit = index == last_index && matches!(*key, "q" | "ctrl-c");
                        if interaction_failed && !is_final_exit {
                            inputs.push(input_record(sequence, key, &[], false, "prior checkpoint failed or was blocked"));
                            continue;
                        }
                        let sent = send_planned_key(&mut *port, &mut tracker, plan, sequence, key, &mut inputs);
                        if !sent {
                            interaction_failed = true;
                        }
                        if is_final_exit {
                            if let Some(sample) = &last_observation {
                                before_exit_modes = Some(sample.modes.clone());
                                before_exit_cursor = Some(sample.cursor.clone());
                            }
                        }
                    }
                }
            }
        } else {
            if let Some(first) = plan.input.iter().find_map(|event| match event {
                InputEvent::WaitFor { checkpoint, .. } => Some(*checkpoint),
                _ => None,
            }) {
                first_frame = Some(Status::Fail);
                checkpoints.push(failed_checkpoint(first, "application was not launched after build".into()));
            }
            interaction_failed = true;
        }

        let first_frame = first_frame.unwrap_or(Status::Blocked);
        set_layer(
            &mut invocation,
            Layer::FirstFrame,
            first_frame,
            if first_frame == Status::Pass { "live PTY grid met the first checkpoint readiness assertions" } else { "first checkpoint frame was not observed" },
            checkpoints.first().and_then(|cp| cp.live_text_sha256.clone()).into_iter().collect(),
        );

        let all_inputs_written = plan.input.iter().filter(|event| matches!(event, InputEvent::Press(_))).count() == inputs.len()
            && inputs.iter().all(|input| input.write_status == "written_and_recorded");
        let capture_ok = !checkpoints.is_empty() && checkpoints.iter().all(|checkpoint| checkpoint.status == Status::Pass);
        let interaction_status = if interaction_failed || !all_inputs_written {
            if checkpoints.iter().any(|checkpoint| checkpoint.status == Status::Fail) { Status::Fail } else { Status::Blocked }
        } else if capture_ok {
            Status::Pass
        } else {
            Status::Blocked
        };
        set_layer(
            &mut invocation,
            Layer::Interaction,
            interaction_status,
            "the command's frozen input and checkpoint program ran in order; negative assertions required earlier positive checkpoints",
            inputs.iter().map(|input| format!("{}:{}:{}", input.sequence, input.key, input.write_status)).collect(),
        );

        let exit_key = plan.input.iter().rev().find_map(|event| match event {
            InputEvent::Press(key) => Some(*key),
            _ => None,
        });
        let exit_sent = inputs.iter().any(|input| {
            Some(input.key.as_str()) == exit_key && input.write_status == "written_and_recorded"
        });
        let exit = match port.wait_exit(Instant::now() + EXIT_TIMEOUT) {
            Ok(exit) => {
                let status = if exit.success && exit.stream == "clean_eof" { Status::Pass } else { Status::Fail };
                set_layer(
                    &mut invocation,
                    Layer::Exit,
                    status,
                    format!("direct child exit code {}; stream {}", exit.code, exit.stream),
                    vec![format!("signal_present={}", exit.signal_present), format!("final_exit_key_sent={exit_sent}")],
                );
                Some(exit)
            }
            Err(error) => {
                set_layer(&mut invocation, Layer::Exit, Status::Blocked,
                    format!("child exit status unavailable: {error}"), vec![format!("final_exit_key_sent={exit_sent}")]);
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
        match sink.store(&format!("{}/{}/{}/launch.raw", plan.command_id, subject.role, invocation_slug(plan.kind)), &full_log) {
            Ok(artifact) => artifact_refs.push(artifact),
            Err(error) => artifact_error = Some(format!("launch log artifact write failed: {error}")),
        }
        if let Some(boundary) = tracker.running_boundary {
            if boundary <= full_log.len() {
                match sink.store(
                    &format!("{}/{}/{}/application.raw", plan.command_id, subject.role, invocation_slug(plan.kind)),
                    &full_log[boundary..],
                ) {
                    Ok(artifact) => artifact_refs.push(artifact),
                    Err(error) => artifact_error = Some(format!("application log artifact write failed: {error}")),
                }
            }
        }
        match serde_json::to_vec_pretty(&inputs)
            .map_err(|error| format!("serialize {} input events: {error}", plan.command_id))
            .and_then(|bytes| sink.store(&format!("{}/{}/{}/inputs.json", plan.command_id, subject.role, invocation_slug(plan.kind)), &bytes))
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
            "required parent/slave termios and shell-continuation restoration are not exposed by this PTY adapter; live child-PTY mode state is supporting evidence only",
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
                before_exit: before_exit_modes,
                after_exit: after_exit_modes,
                before_exit_cursor: before_exit_cursor,
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

    fn blocked_checkpoint(id: &str, reason: String) -> CheckpointReceipt {
        CheckpointReceipt {
            id: id.to_string(),
            status: Status::Blocked,
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

    fn pair_environment_matches(reference: &EnvironmentReceipt, candidate: &EnvironmentReceipt) -> bool {
        let mut reference_inputs = reference.variable_hashes.clone();
        let mut candidate_inputs = candidate.variable_hashes.clone();
        // Target directories are build outputs and may be isolated per source
        // subject. All inherited process inputs and app-owned values stay the
        // same for the pair.
        reference_inputs.remove("CARGO_TARGET_DIR");
        candidate_inputs.remove("CARGO_TARGET_DIR");
        reference_inputs == candidate_inputs
            && reference.case_environment_hashes == candidate.case_environment_hashes
            && reference.cargo_path == candidate.cargo_path
            && reference.cargo_sha256 == candidate.cargo_sha256
            && reference.rustc_path == candidate.rustc_path
            && reference.rustc_sha256 == candidate.rustc_sha256
            && reference.target_triple == candidate.target_triple
            && reference.cargo_jobs == candidate.cargo_jobs
            && reference.inherited_environment == candidate.inherited_environment
            && reference.cargo_runner == candidate.cargo_runner
    }

    pub fn execute_command_pair(
        command_id: &str,
        subjects: &[FrozenCommandSubject],
        environments: &BTreeMap<String, ExecutionEnvelope>,
        factory: &dyn PtyFactory,
        recorder: &mut dyn FrameRecorder,
        sink: &mut DirectoryArtifactSink,
    ) -> Result<(CommandReceipt, Vec<RunEvidence>), String> {
        let row_index = command_row_index(command_id)?;
        if !matches!(std::env::consts::OS, "linux" | "macos") {
            return Err(format!("{command_id} PTY execution is blocked on unverified platform {}", std::env::consts::OS));
        }
        let program = command_program(command_id)
            .ok_or_else(|| format!("{command_id} has no shared command program"))?;
        validate_command_program(program)?;
        let mut selections = BTreeMap::new();
        for subject in subjects {
            selections.insert(subject.role, native_command_selection(command_id, subject)?);
        }
        let selection = selections.values().next()
            .ok_or_else(|| "command pair has no selected subjects".to_string())?;
        if selections.values().any(|item| item.cargo_argv != selection.cargo_argv
            || item.app_args != selection.app_args
            || item.source_citations != selection.source_citations)
        {
            return Err("reference and candidate command selections differ".into());
        }
        let mut receipt = initial_command_receipt(subjects, command_id)?;
        let mut evidence = Vec::new();
        let protected_roots: Vec<PathBuf> = subjects.iter().map(|subject| subject.source_root.clone()).collect();
        let mut validated = BTreeMap::new();
        let mut preflight_failure = None;
        for subject in subjects {
            match environments.get(subject.role)
                .ok_or_else(|| format!("missing validated execution envelope for {}", subject.role))
                .and_then(|envelope| {
                    let selected = selections.get(subject.role)
                        .ok_or_else(|| format!("missing selected command for {}", subject.role))?;
                    let plan = command_plans(command_id, subject)?;
                    envelope.validate(subject, selected.binary_name, &plan[0].case_environment, &protected_roots)
                        .map(|receipt| (envelope, receipt))
                })
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
        if preflight_failure.is_none() {
            let reference = validated.get("reference").map(|(_, receipt)| receipt);
            let candidate = validated.get("candidate").map(|(_, receipt)| receipt);
            if !matches!((reference, candidate), (Some(reference), Some(candidate)) if pair_environment_matches(reference, candidate)) {
                preflight_failure = Some("reference and candidate do not share the same controlled command environment".into());
            }
        }
        if let Some(error) = preflight_failure {
            let row = receipt.rows.get_mut(row_index).ok_or_else(|| "selected command receipt row missing".to_string())?;
            for invocation in row.reference.iter_mut().chain(row.candidate.iter_mut())
                .chain(row.direct_executable_diagnostics.iter_mut())
            {
                for layer in [Layer::Build, Layer::Launch, Layer::FirstFrame, Layer::Interaction, Layer::Exit, Layer::Cleanup, Layer::Restoration, Layer::Visual, Layer::Ownership] {
                    let status = match (invocation.kind, layer, invocation.role) {
                        (InvocationKind::DirectExecutableDiagnostic, Layer::Build, _) => Status::NotApplicable,
                        (_, Layer::Ownership, "reference") => Status::NotApplicable,
                        _ => Status::Blocked,
                    };
                    let reason = match (invocation.kind, layer, invocation.role) {
                        (InvocationKind::DirectExecutableDiagnostic, Layer::Build, _) =>
                            "direct executable diagnostic does not invoke Cargo".to_string(),
                        (_, Layer::Ownership, "reference") =>
                            "reference implementation is not required to use candidate component ownership".to_string(),
                        _ => format!("pair preflight blocked before any process launched: {error}"),
                    };
                    set_layer(invocation, layer, status, reason, vec![]);
                }
            }
            row.status = Status::Blocked;
            return Ok((receipt, evidence));
        }
        for subject in subjects {
            let (envelope, env_receipt) = validated.get(subject.role)
                .ok_or_else(|| format!("validated environment missing for {}", subject.role))?;
            let plans = command_plans(command_id, subject)?;
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
                            before_exit: None, after_exit: None,
                            before_exit_cursor: None, after_exit_cursor: None,
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
                                "frozen selected executable bytes changed during the invocation", vec![
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
                                before_exit: None, after_exit: None,
                                before_exit_cursor: None, after_exit_cursor: None,
                                host_terminal_state: "not_observed".into(),
                                child_pty_state: "not_observed".into(),
                            }, artifacts: Vec::new(), cleanup: cleanup_not_run(),
                        }
                    }
                    }
                };
                let row = receipt.rows.get_mut(row_index).ok_or_else(|| "selected command receipt row missing".to_string())?;
                let target = match (subject.role, plan.kind) {
                    ("reference", InvocationKind::ExactRootCommand) => row.reference.as_mut(),
                    ("candidate", InvocationKind::ExactRootCommand) => row.candidate.as_mut(),
                    ("reference", InvocationKind::DirectExecutableDiagnostic) => row.direct_executable_diagnostics.iter_mut().find(|item| item.role == "reference"),
                    ("candidate", InvocationKind::DirectExecutableDiagnostic) => row.direct_executable_diagnostics.iter_mut().find(|item| item.role == "candidate"),
                    _ => None,
                }.ok_or_else(|| format!("{command_id} invocation slot missing"))?;
                *target = invocation.invocation.clone();
                evidence.push(invocation);
            }
        }
        let row = receipt.rows.get_mut(row_index).ok_or_else(|| "selected command receipt row missing".to_string())?;
        row.status = aggregate_command_status(row);
        Ok((receipt, evidence))
    }

    pub fn execute_run03_pair(
        subjects: &[FrozenCommandSubject],
        environments: &BTreeMap<String, ExecutionEnvelope>,
        factory: &dyn PtyFactory,
        recorder: &mut dyn FrameRecorder,
        sink: &mut DirectoryArtifactSink,
    ) -> Result<(CommandReceipt, Vec<RunEvidence>), String> {
        execute_command_pair("RUN-03", subjects, environments, factory, recorder, sink)
    }

    /// Dispatch by stable RUN ID. Every row uses the same generic PTY runner,
    /// the command-specific source-backed program, and the independently
    /// resolved executable for each frozen subject.
    pub fn dispatch_native_command_pair(
        command_id: &str,
        subjects: &[FrozenCommandSubject],
        environments: &BTreeMap<String, ExecutionEnvelope>,
        factory: &dyn PtyFactory,
        recorder: &mut dyn FrameRecorder,
        sink: &mut DirectoryArtifactSink,
    ) -> Result<(CommandReceipt, Vec<RunEvidence>), String> {
        let row_index = command_row_index(command_id)?;
        let selections = subjects.iter()
            .map(|subject| native_command_selection(command_id, subject))
            .collect::<Result<Vec<_>, _>>()?;
        let blocked_reason = selections.iter().find_map(|selection| {
            match (&selection.program_readiness, &selection.execution_readiness) {
                (CommandProgramReadiness::Blocked { reason }, _) => Some(reason.clone()),
                (_, CommandExecutionReadiness::Blocked { reason }) => Some(reason.clone()),
                _ => None,
            }
        });
        if let Some(reason) = blocked_reason {
            let mut receipt = initial_command_receipt(subjects, command_id)?;
            let reason = format!("{command_id} is blocked before spawn: {reason}");
            for (subject, selection) in subjects.iter().zip(selections.iter()) {
            let root = blocked_program_invocation(
                subject,
                InvocationKind::ExactRootCommand,
                selection.cargo_argv.clone(),
                &reason,
            );
            let direct = blocked_program_invocation(
                subject,
                InvocationKind::DirectExecutableDiagnostic,
                selection.direct_argv.clone(),
                &reason,
            );
            let row = &mut receipt.rows[row_index];
            match subject.role {
                "reference" => row.reference = Some(root),
                "candidate" => row.candidate = Some(root),
                other => return Err(format!("unsupported subject role {other:?}")),
            }
            if let Some(slot) = row
                .direct_executable_diagnostics
                .iter_mut()
                .find(|slot| slot.role == subject.role)
            {
                *slot = direct;
            } else {
                row.direct_executable_diagnostics.push(direct);
            }
            }
            receipt.rows[row_index].status = Status::Blocked;
            return Ok((receipt, Vec::new()));
        }
        execute_command_pair(command_id, subjects, environments, factory, recorder, sink)
    }

    fn blocked_program_invocation(
        subject: &FrozenCommandSubject,
        kind: InvocationKind,
        argv: Vec<String>,
        reason: &str,
    ) -> InvocationReceipt {
        let mut invocation = empty_invocation(subject, kind, argv);
        for layer in [
            Layer::Build,
            Layer::Launch,
            Layer::FirstFrame,
            Layer::Interaction,
            Layer::Exit,
            Layer::Cleanup,
            Layer::Restoration,
            Layer::Visual,
        ] {
            set_layer(&mut invocation, layer, Status::Blocked, reason, Vec::new());
        }
        if kind == InvocationKind::DirectExecutableDiagnostic {
            set_layer(
                &mut invocation,
                Layer::Build,
                Status::NotApplicable,
                "direct executable diagnostic does not invoke Cargo",
                Vec::new(),
            );
        }
        let (status, ownership_reason) = if subject.role == "reference" {
            (
                Status::NotApplicable,
                "reference implementation is outside candidate ownership applicability",
            )
        } else {
            (
                Status::Blocked,
                "candidate ownership was not observed because the command was not launched",
            )
        };
        set_layer(
            &mut invocation,
            Layer::Ownership,
            status,
            ownership_reason,
            Vec::new(),
        );
        invocation
    }

    fn aggregate_command_status(row: &CommandRow) -> Status {
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
            fn capture(&mut self, _command_id: &str, _role: &str, _kind: InvocationKind, checkpoint: &str,
                sample: &LiveSample, _recording: &tuiscotti::tui_shell::Recording,
                _sink: &mut dyn ArtifactSink) -> Result<CaptureProof, String> {
                Ok(CaptureProof {
                    artifacts: vec![], frame_digest: checkpoint.into(),
                    replay_text_matches_live: true, replay_cursor_matches_live: true,
                })
            }
        }

        struct NoSpawnFactory(Arc<AtomicBool>);
        impl PtyFactory for NoSpawnFactory {
            fn spawn(
                &self,
                _subject: &FrozenCommandSubject,
                _plan: &InvocationPlan,
                _env: &ExecutionEnvelope,
            ) -> Result<Box<dyn CommandPty>, String> {
                self.0.store(true, Ordering::SeqCst);
                Err("unsupported command unexpectedly reached PTY spawn".into())
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
            live_sample_for_executable(
                revision,
                text,
                Path::new("/frozen/ref/target/release/holla"),
            )
        }

        fn live_sample_for_executable(
            revision: u64,
            text: &str,
            executable: &Path,
        ) -> LiveSample {
            let log = format!(
                "    Finished `release` profile [optimized] target(s) in 0.00s\n   Running `{}`\n{text}",
                executable.display()
            )
            .into_bytes();
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
            assert_eq!(INPUT_CTRL_N, b"\x0e");
            assert_eq!(INPUT_DOWN, b"\x1b[B");
            assert_eq!(INPUT_ENTER, b"\r");
            assert_eq!(INPUT_CTRL_D, b"\x04");
            assert_eq!(INPUT_NEXT_PAGE, b"]");
            assert_eq!(INPUT_QUIT, b"q");
        }

        #[test]
        fn planned_key_encoder_requires_a_declared_press_and_rejects_unknown_keys() {
            let mut plan = root_plan();
            assert_eq!(planned_key_bytes(&plan, "ctrl-n"), None);
            assert_eq!(planned_key_bytes(&plan, "not-a-key"), None);

            let keys = ["ctrl-n", "down", "enter", "ctrl-d", "]", "q"];
            plan.input
                .extend(keys.into_iter().map(InputEvent::Press));
            let expected: [(&str, &[u8]); 6] = [
                ("ctrl-n", b"\x0e"),
                ("down", b"\x1b[B"),
                ("enter", b"\r"),
                ("ctrl-d", b"\x04"),
                ("]", b"]"),
                ("q", b"q"),
            ];
            for (key, bytes) in expected {
                assert_eq!(planned_key_bytes(&plan, key), Some(bytes));
            }
            plan.input.retain(|event| !matches!(event, InputEvent::Press(key) if *key == "enter"));
            assert_eq!(planned_key_bytes(&plan, "enter"), None);
            assert_eq!(planned_key_bytes(&plan, "not-a-key"), None);
        }

        #[test]
        fn every_frozen_command_program_uses_supported_declared_key_encodings() {
            for (id, _, argv) in COMMANDS {
                let binary = argv.windows(2).find(|pair| pair[0] == "--bin").unwrap()[1];
                let subject = FrozenCommandSubject {
                    role: "reference",
                    source_manifest: PathBuf::from("/frozen/reference/manifest.json"),
                    source_manifest_sha256: "a".repeat(64),
                    source_root: PathBuf::from("/frozen/reference/tree"),
                    source_commit: "b".repeat(40),
                    toolchain: "stable-x86_64-unknown-linux-gnu".into(),
                    target_triple: "x86_64-unknown-linux-gnu".into(),
                    executable: PathBuf::from(format!("/frozen/reference/target/release/{binary}")),
                    executable_sha256: "c".repeat(64),
                };
                let plan = command_plans(id, &subject).unwrap()[0].clone();
                for key in plan.input.iter().filter_map(|event| match event {
                    InputEvent::Press(key) => Some(*key),
                    InputEvent::WaitFor { .. } => None,
                }) {
                    assert!(planned_key_bytes(&plan, key).is_some(), "{id} has unsupported key {key}");
                }
            }
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
        fn shared_root_command_driver_runs_all_programs_against_the_selected_target() {
            for (id, _, argv) in COMMANDS {
                let binary = argv.windows(2).find(|pair| pair[0] == "--bin").unwrap()[1];
                let subject = FrozenCommandSubject {
                    role: "reference",
                    source_manifest: PathBuf::from("/frozen/reference/manifest.json"),
                    source_manifest_sha256: "a".repeat(64),
                    source_root: PathBuf::from("/frozen/reference/tree"),
                    source_commit: "b".repeat(40),
                    toolchain: "stable-x86_64-unknown-linux-gnu".into(),
                    target_triple: "x86_64-unknown-linux-gnu".into(),
                    executable: PathBuf::from(format!("/frozen/reference/target/release/{binary}")),
                    executable_sha256: "c".repeat(64),
                };
                let plan = command_plans(id, &subject).unwrap()[0].clone();
                let program = command_program(id).unwrap();
                let samples = program
                    .input
                    .iter()
                    .filter_map(|event| match event {
                        InputEvent::WaitFor { contains, .. } => Some(contains.join("\n")),
                        InputEvent::Press(_) => None,
                    })
                    .enumerate()
                    .map(|(index, text)| {
                        live_sample_for_executable(
                            (index + 1) as u64,
                            &text,
                            &subject.executable,
                        )
                    })
                    .collect();
                let (port, closed) = FakePty::new(
                    samples,
                    Ok(ExitRecord {
                        success: true,
                        code: 0,
                        signal_present: false,
                        stream: "clean_eof".into(),
                    }),
                    Ok(()),
                );
                let outcome = drive_invocation(
                    &subject,
                    &plan,
                    None,
                    Box::new(port),
                    &mut FakeRecorder,
                    &mut MemorySink,
                );
                assert_eq!(outcome.invocation.layers[&Layer::Build].status, Status::Pass, "{id}");
                assert_eq!(outcome.invocation.layers[&Layer::Launch].status, Status::Pass, "{id}");
                assert_eq!(outcome.invocation.layers[&Layer::FirstFrame].status, Status::Pass, "{id}");
                assert_eq!(outcome.invocation.layers[&Layer::Interaction].status, Status::Pass, "{id}");
                assert_eq!(outcome.invocation.layers[&Layer::Exit].status, Status::Pass, "{id}");
                assert_eq!(outcome.invocation.layers[&Layer::Cleanup].status, Status::Pass, "{id}");
                assert_eq!(outcome.invocation.layers[&Layer::Restoration].status, Status::Blocked, "{id}");
                assert_eq!(outcome.invocation.layers[&Layer::Visual].status, Status::Blocked, "{id}");
                assert_eq!(outcome.checkpoints.len(), program.input.iter().filter(|event| matches!(event, InputEvent::WaitFor { .. })).count(), "{id}");
                assert!(closed.load(Ordering::SeqCst), "{id}");
            }
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

        #[test]
        fn missing_execution_envelope_blocks_the_selected_program_without_spawning() {
            let spawned = Arc::new(AtomicBool::new(false));
            for (id, _, argv) in COMMANDS {
                let binary = argv.windows(2).find(|pair| pair[0] == "--bin").unwrap()[1];
                let subjects = [
                    FrozenCommandSubject {
                        role: "reference",
                        source_manifest: PathBuf::from("/frozen/reference/manifest.json"),
                        source_manifest_sha256: "a".repeat(64),
                        source_root: PathBuf::from("/frozen/reference/tree"),
                        source_commit: "b".repeat(40),
                        toolchain: "stable-x86_64-unknown-linux-gnu".into(),
                        target_triple: "x86_64-unknown-linux-gnu".into(),
                        executable: PathBuf::from(format!("/frozen/reference/target/release/{binary}")),
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
                        executable: PathBuf::from(format!("/frozen/candidate/target/release/{binary}")),
                        executable_sha256: "f".repeat(64),
                    },
                ];
                let (receipt, evidence) = dispatch_native_command_pair(
                    id,
                    &subjects,
                    &BTreeMap::new(),
                    &NoSpawnFactory(spawned.clone()),
                    &mut FakeRecorder,
                    &mut DirectoryArtifactSink { root: PathBuf::from("/unused/no-artifact-write") },
                )
                .unwrap();
                let row_index = command_row_index(id).unwrap();
                let row = &receipt.rows[row_index];
                assert_eq!(row.status, Status::Blocked, "{id}");
                let expected_root_argv = argv.iter().map(|part| (*part).to_owned()).collect::<Vec<_>>();
                assert_eq!(row.reference.as_ref().unwrap().argv, expected_root_argv, "{id}");
                assert_eq!(row.candidate.as_ref().unwrap().argv, expected_root_argv, "{id}");
                assert_eq!(row.direct_executable_diagnostics.len(), 2, "{id}");
                let mut expected_direct = vec![format!("/frozen/reference/target/release/{binary}")];
                if let Some(separator) = argv.iter().position(|arg| *arg == "--") {
                    expected_direct.extend(argv[separator + 1..].iter().map(|part| (*part).to_owned()));
                }
                assert_eq!(row.direct_executable_diagnostics[0].argv, expected_direct, "{id}");
                assert!(receipt.rows.iter().enumerate().all(|(index, other)| {
                    index == row_index || (other.status == Status::NotRun && other.direct_executable_diagnostics.is_empty())
                }));
                assert!(evidence.is_empty(), "{id}");
            }
            assert!(!spawned.load(Ordering::SeqCst));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    use executor::{ArtifactSink, DirectoryArtifactSink, ExecutionEnvelope, TermpaneFactory, TuiscottiRecorder};
    #[cfg(unix)]
    use sha2::{Digest, Sha256};
    #[cfg(unix)]
    use std::collections::BTreeSet;
    #[cfg(unix)]
    use std::fs;
    #[cfg(unix)]
    use std::path::PathBuf;
    #[cfg(unix)]
    use termrock_e2e::COMPILED_SUITE_SHA256;

    #[cfg(unix)]
    const RUN03_CASE_PATH_ENV: &str = "TERMROCK_E2E_RUN03_PAIR_CASE_PATH";
    #[cfg(unix)]
    const RUN03_CASE_SHA_ENV: &str = "TERMROCK_E2E_RUN03_PAIR_CASE_SHA256";
    #[cfg(unix)]
    const RUN03_CASE_SCHEMA: &str = "termrock-e2e/run03-pair-case-v1";
    #[cfg(unix)]
    const RUN03_CASE_MAX_BYTES: u64 = 512 * 1024;
    #[cfg(unix)]
    const RUN03_REFERENCE_COMMIT: &str = "b274dd57f4dd078ade6e424d546d83efbd2e8526";
    #[cfg(unix)]
    const RUN03_CANDIDATE_COMMIT: &str = "1d797d41c8141fcbdc3f69d7f11eb8875ab54712";

    #[cfg(unix)]
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Run03PairCase {
        schema: String,
        compiled_suite_sha256: String,
        artifact_root: PathBuf,
        subjects: Vec<Run03Subject>,
        environments: BTreeMap<String, ExecutionEnvelope>,
    }

    #[cfg(unix)]
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Run03Subject {
        role: String,
        source_manifest: PathBuf,
        source_manifest_sha256: String,
        source_root: PathBuf,
        source_commit: String,
        toolchain: String,
        target_triple: String,
        executable: PathBuf,
        executable_sha256: String,
    }

    #[cfg(unix)]
    impl Run03Subject {
        fn into_frozen(self) -> FrozenCommandSubject {
            let role = match self.role.as_str() {
                "reference" => "reference",
                "candidate" => "candidate",
                other => panic!("unsupported RUN03 subject role {other:?}"),
            };
            FrozenCommandSubject {
                role,
                source_manifest: self.source_manifest,
                source_manifest_sha256: self.source_manifest_sha256,
                source_root: self.source_root,
                source_commit: self.source_commit,
                toolchain: self.toolchain,
                target_triple: self.target_triple,
                executable: self.executable,
                executable_sha256: self.executable_sha256,
            }
        }
    }

    #[cfg(unix)]
    fn read_run03_case() -> (Run03PairCase, String) {
        let path = std::env::var_os(RUN03_CASE_PATH_ENV)
            .map(PathBuf::from)
            .unwrap_or_else(|| panic!("{RUN03_CASE_PATH_ENV} is required; refusing to launch without a frozen pair case"));
        assert!(path.is_absolute(), "RUN03 pair case path must be absolute");
        let metadata = fs::symlink_metadata(&path)
            .unwrap_or_else(|error| panic!("inspect RUN03 pair case {}: {error}", path.display()));
        assert!(metadata.file_type().is_file(), "RUN03 pair case must be a regular non-symlink file");
        assert!(metadata.len() <= RUN03_CASE_MAX_BYTES, "RUN03 pair case exceeds the 512 KiB input bound");
        let bytes = fs::read(&path)
            .unwrap_or_else(|error| panic!("read RUN03 pair case {}: {error}", path.display()));
        assert!(bytes.len() as u64 <= RUN03_CASE_MAX_BYTES, "RUN03 pair case grew beyond the 512 KiB input bound");
        let expected_sha = std::env::var(RUN03_CASE_SHA_ENV)
            .unwrap_or_else(|_| panic!("{RUN03_CASE_SHA_ENV} is required; refusing to use an unpinned pair case"));
        assert!(expected_sha.len() == 64 && expected_sha.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()),
            "RUN03 pair case SHA-256 must be lowercase hexadecimal");
        let actual_sha = format!("{:x}", Sha256::digest(&bytes));
        assert_eq!(actual_sha, expected_sha, "RUN03 pair case bytes differ from their supplied digest");
        let case = serde_json::from_slice::<Run03PairCase>(&bytes)
            .unwrap_or_else(|error| panic!("parse strict RUN03 pair case {}: {error}", path.display()));
        (case, actual_sha)
    }

    #[cfg(unix)]
    fn assert_run03_layer(run: &executor::RunEvidence, layer: Layer, expected: Status) {
        assert_eq!(run.invocation.layers[&layer].status, expected,
            "{} {:?} {:?}: {}", run.invocation.role, run.invocation.kind, layer,
            run.invocation.layers[&layer].reason);
    }

    fn command_subject(role: &'static str, binary: &str) -> FrozenCommandSubject {
        FrozenCommandSubject {
            role,
            source_manifest: PathBuf::from(format!("/frozen/{role}/manifest.json")),
            source_manifest_sha256: "a".repeat(64),
            source_root: PathBuf::from(format!("/frozen/{role}/tree")),
            source_commit: "b".repeat(40),
            toolchain: "stable-x86_64-unknown-linux-gnu".into(),
            target_triple: "x86_64-unknown-linux-gnu".into(),
            executable: PathBuf::from(format!("/frozen/{role}/target/release/{binary}")),
            executable_sha256: "c".repeat(64),
        }
    }

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
        let subject = command_subject("reference", "holla");
        let plans = run03_plans(&subject).unwrap();
        assert_eq!(plans[0].argv, ["cargo", "run", "--release", "--bin", "holla"]);
        assert_eq!(plans[0].cwd, subject.source_root);
        assert!(!plans[0].argv.iter().any(|arg| arg == "--scenario" || arg == "paused"));
        assert_eq!(plans[1].argv, ["/frozen/reference/target/release/holla"]);
        assert_eq!(plans[1].kind, InvocationKind::DirectExecutableDiagnostic);
    }

    #[test]
    fn all_run03_invocations_share_one_input_program_and_geometry() {
        let subject = command_subject("candidate", "holla");
        let candidate = run03_plans(&subject).unwrap();
        let reference = run03_plans(&command_subject("reference", "holla")).unwrap();
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
        assert!(receipt.rows.iter().all(|row| row.direct_executable_diagnostics.is_empty()));
    }

    #[test]
    fn lane_declares_resource_limits_and_native_platform_boundary() {
        assert_eq!(MAX_CONCURRENT_PTYS, 2);
        assert_eq!(CARGO_BUILD_JOBS, 2);
        assert_eq!(platform_eligibility("linux"), PlatformEligibility::SupportedLinuxOrMacos);
        assert_eq!(platform_eligibility("macos"), PlatformEligibility::SupportedLinuxOrMacos);
        assert_eq!(platform_eligibility("windows"), PlatformEligibility::BlockedUnsupportedOrUnverified);
    }

    #[test]
    fn native_command_selectors_preserve_all_exact_targets_and_app_arguments() {
        let cases = [
            ("RUN-01", "showcase", vec![]),
            ("RUN-02", "jackin-preview", vec![]),
            ("RUN-03", "holla", vec![]),
            ("RUN-04", "tablepro", vec![]),
            ("RUN-05", "tablepro", vec!["--connect", "Production"]),
            ("RUN-06", "jackin-preview", vec!["--scenario", "accounts-mixed"]),
            ("RUN-07", "holla", vec!["--scenario", "remote-host"]),
        ];
        assert_eq!(cases.len(), COMMANDS.len());
        for (expected_index, (id, binary, app_args)) in cases.into_iter().enumerate() {
            let selection = native_command_selection(id, &command_subject("reference", binary)).unwrap();
            let (registered_id, exact_command, argv) = COMMANDS[expected_index];
            assert_eq!(selection.id, registered_id);
            assert_eq!(selection.id, id);
            assert_eq!(command_row_index(id).unwrap(), expected_index);
            assert_eq!(selection.exact_command, exact_command);
            assert_eq!(
                selection.cargo_argv,
                argv.iter().map(|part| (*part).to_owned()).collect::<Vec<_>>()
            );
            assert_eq!(selection.binary_name, binary);
            assert_eq!(
                selection.app_args,
                app_args.iter().map(|part| (*part).to_owned()).collect::<Vec<_>>()
            );
            let mut expected_direct = vec![format!("/frozen/reference/target/release/{binary}")];
            expected_direct.extend(app_args.iter().map(|part| (*part).to_owned()));
            assert_eq!(selection.direct_argv, expected_direct);
        }
    }

    #[test]
    fn all_seven_programs_are_valid_source_backed_and_role_independent() {
        assert_eq!(COMMAND_PROGRAMS.len(), COMMANDS.len());
        for (id, _, argv) in COMMANDS {
            let binary = argv
                .windows(2)
                .find(|pair| pair[0] == "--bin")
                .map(|pair| pair[1])
                .unwrap();
            let program = command_program(id).expect("every exact root command has a program");
            validate_command_program(program).unwrap();
            assert_eq!(program.command_id, id);
            assert!(!program.source_citations.is_empty());
            if id == "RUN-03" {
                assert_eq!(program.case_environment, &[("HOLLA_NO_HISTORY", "1")]);
            } else {
                assert!(program.case_environment.is_empty(), "{id} must not inherit a Holla-only override");
            }

            let candidate = command_subject("candidate", binary);
            let reference = command_subject("reference", binary);
            let candidate_selection = native_command_selection(id, &candidate).unwrap();
            let reference_selection = native_command_selection(id, &reference).unwrap();
            assert_eq!(candidate_selection.program_readiness, CommandProgramReadiness::Ready);
            assert_eq!(reference_selection.program_readiness, CommandProgramReadiness::Ready);
            assert_eq!(candidate_selection.execution_readiness, reference_selection.execution_readiness);
            assert_eq!(candidate_selection.source_citations, reference_selection.source_citations);
            assert_eq!(candidate_selection.cargo_argv, reference_selection.cargo_argv);
            assert_eq!(candidate_selection.app_args, reference_selection.app_args);

            let candidate_plans = command_plans(id, &candidate).unwrap();
            let reference_plans = command_plans(id, &reference).unwrap();
            assert_eq!(candidate_plans[0].input.as_slice(), program.input);
            assert_eq!(candidate_plans[1].input.as_slice(), program.input);
            assert_eq!(reference_plans[0].input, candidate_plans[0].input);
            assert_eq!(reference_plans[1].input, candidate_plans[1].input);
            assert_eq!(candidate_plans[0].case_environment, reference_plans[0].case_environment);
            assert_eq!(candidate_plans[1].case_environment, candidate_plans[0].case_environment);
            assert_eq!(candidate_plans[0].argv, candidate_selection.cargo_argv);
            assert_eq!(reference_plans[0].argv, reference_selection.cargo_argv);
            assert_eq!(candidate_plans[1].argv, candidate_selection.direct_argv);
            assert_eq!(candidate_plans[0].command_id, id);
            assert_eq!(candidate_plans[1].command_id, id);
            assert_eq!(candidate_plans[0].size, (120, 40));
            assert_eq!(candidate_plans[0].max_concurrent_ptys, 2);
            assert_eq!(candidate_plans[0].cargo_build_jobs, 2);
        }
    }

    #[test]
    fn command_programs_keep_expected_checkpoint_and_input_sequences() {
        let expected: [(&str, &[&str], &[&str]); 7] = [
            ("RUN-01", &["00-overview", "01-buttons"], &["]", "q"]),
            ("RUN-02", &["00-intro", "01-manager", "02-new-workspace"], &["down", "q"]),
            ("RUN-03", &["00-root", "01-help", "02-root-restored"], &["f1", "escape", "ctrl-c"]),
            ("RUN-04", &["00-connections", "01-new-connection-form", "02-connections-restored"], &["ctrl-n", "escape", "ctrl-c"]),
            ("RUN-05", &["00-explorer", "01-order-detail", "02-foreign-keys"], &["down", "down", "down", "down", "down", "enter", "ctrl-d", "ctrl-c"]),
            ("RUN-06", &["00-accounts", "01-work-account-drawer", "02-manager-restored"], &["down", "down", "down", "enter", "escape", "escape", "q"]),
            ("RUN-07", &["00-remote-host", "01-help", "02-remote-host-restored"], &["f1", "escape", "ctrl-c"]),
        ];
        for (id, checkpoint_ids, keys) in expected {
            let program = command_program(id).unwrap();
            let actual_checkpoints = program.input.iter().filter_map(|event| match event {
                InputEvent::WaitFor { checkpoint, .. } => Some(*checkpoint),
                InputEvent::Press(_) => None,
            }).collect::<Vec<_>>();
            let actual_keys = program.input.iter().filter_map(|event| match event {
                InputEvent::WaitFor { .. } => None,
                InputEvent::Press(key) => Some(*key),
            }).collect::<Vec<_>>();
            assert_eq!(actual_checkpoints, checkpoint_ids, "{id} checkpoint order");
            assert_eq!(actual_keys, keys, "{id} key order");
        }
    }

    #[test]
    fn program_contract_rejects_vacuous_or_unpaired_negative_checkpoints() {
        let vacuous = CommandProgram {
            command_id: "TEST",
            input: &[
                InputEvent::WaitFor { checkpoint: "00", contains: &[], absent: &[], requires: &[] },
                InputEvent::Press("q"),
            ],
            case_environment: &[],
            source_citations: &["synthetic test only"],
        };
        assert!(validate_command_program(&vacuous).unwrap_err().contains("positive readiness"));

        let unpaired_negative = CommandProgram {
            command_id: "TEST",
            input: &[
                InputEvent::WaitFor { checkpoint: "00", contains: &["ready"], absent: &[], requires: &[] },
                InputEvent::WaitFor { checkpoint: "01", contains: &["restored"], absent: &["dialog"], requires: &[] },
                InputEvent::Press("q"),
            ],
            case_environment: &[],
            source_citations: &["synthetic test only"],
        };
        assert!(validate_command_program(&unpaired_negative).unwrap_err().contains("earlier positive prerequisite"));
    }

    #[test]
    fn native_command_selector_rejects_unknown_ids_and_wrong_binary_identity() {
        let subject = command_subject("reference", "showcase");
        assert!(native_command_selection("RUN-08", &subject).unwrap_err().contains("unknown native command ID"));
        assert!(native_command_selection("RUN-02", &subject).unwrap_err().contains("selects binary"));
        assert_eq!(command_row_index("RUN-07").unwrap(), 6);
        assert!(command_row_index("RUN-08").unwrap_err().contains("unknown native command ID"));
    }

    #[cfg(unix)]
    #[test]
    #[ignore = "requires a SHA-pinned frozen reference/candidate pair case and Root-authorized PTY launch"]
    fn run03_frozen_holla_pair_uses_the_shared_native_pty_program() {
        let (case, case_sha256) = read_run03_case();
        assert_eq!(case.schema, RUN03_CASE_SCHEMA, "unsupported RUN03 pair-case schema");
        assert_eq!(case.compiled_suite_sha256, COMPILED_SUITE_SHA256,
            "RUN03 pair case was prepared for a different compiled shared-suite revision");
        assert_eq!(case.subjects.len(), 2, "RUN03 requires exactly one frozen reference and candidate");

        let roles = case.subjects.iter().map(|subject| subject.role.as_str()).collect::<BTreeSet<_>>();
        assert_eq!(roles, BTreeSet::from(["candidate", "reference"]),
            "RUN03 pair case must bind exactly the reference and candidate roles");
        assert_eq!(case.environments.keys().map(String::as_str).collect::<BTreeSet<_>>(), roles,
            "RUN03 pair case must supply one execution envelope for each frozen role");

        let subjects = case.subjects.into_iter().map(Run03Subject::into_frozen).collect::<Vec<_>>();
        for subject in &subjects {
            assert_eq!(subject.source_manifest, subject.source_root.join("Cargo.toml"),
                "{} manifest must be the pinned source root manifest", subject.role);
            let expected_commit = match subject.role {
                "reference" => RUN03_REFERENCE_COMMIT,
                "candidate" => RUN03_CANDIDATE_COMMIT,
                other => panic!("unsupported frozen RUN03 role {other:?}"),
            };
            assert_eq!(subject.source_commit, expected_commit, "{} source commit changed", subject.role);
            assert_eq!(case.environments[subject.role].artifact_root, case.artifact_root,
                "{} execution envelope and paired artifact root differ", subject.role);
        }

        let protected_roots = subjects.iter().map(|subject| subject.source_root.clone()).collect::<Vec<_>>();
        let mut sink = DirectoryArtifactSink::new(&case.artifact_root, &protected_roots)
            .unwrap_or_else(|error| panic!("validate RUN03 external artifact root: {error}"));
        let (receipt, evidence) = executor::execute_run03_pair(
            &subjects,
            &case.environments,
            &TermpaneFactory,
            &mut TuiscottiRecorder,
            &mut sink,
        ).unwrap_or_else(|error| panic!("RUN03 preflight failed before pair execution: {error}"));

        let output = serde_json::json!({
            "schema": "termrock-e2e/run03-pair-evidence-v1",
            "case_sha256": case_sha256,
            "compiled_suite_sha256": COMPILED_SUITE_SHA256,
            "receipt": &receipt,
            "runs": &evidence,
        });
        let output_bytes = serde_json::to_vec_pretty(&output).expect("serialize RUN03 pair evidence");
        sink.store("RUN-03/pair-evidence.json", &output_bytes)
            .unwrap_or_else(|error| panic!("persist RUN03 pair evidence before assertions: {error}"));

        assert_eq!(evidence.len(), 4, "RUN03 must retain both root-command and direct-executable invocations per role");
        let expected_kinds = [InvocationKind::ExactRootCommand, InvocationKind::DirectExecutableDiagnostic];
        for role in ["reference", "candidate"] {
            let subject = subjects.iter().find(|subject| subject.role == role).unwrap();
            for kind in expected_kinds {
                let mut matching = evidence.iter().filter(|run| run.invocation.role == role && run.invocation.kind == kind);
                let run = matching.next().unwrap_or_else(|| panic!("missing RUN03 {role} {kind:?} evidence"));
                assert!(matching.next().is_none(), "duplicate RUN03 {role} {kind:?} evidence");
                let expected_argv = match kind {
                    InvocationKind::ExactRootCommand => vec!["cargo", "run", "--release", "--bin", "holla"]
                        .into_iter().map(str::to_owned).collect::<Vec<_>>(),
                    InvocationKind::DirectExecutableDiagnostic => vec![subject.executable.display().to_string()],
                };
                assert_eq!(run.invocation.argv, expected_argv, "RUN03 command argv changed");
                assert_eq!(run.invocation.cwd, subject.source_root, "RUN03 command cwd changed");
                assert_run03_layer(run, Layer::Build, if kind == InvocationKind::ExactRootCommand { Status::Pass } else { Status::NotApplicable });
                for layer in [Layer::Launch, Layer::FirstFrame, Layer::Interaction, Layer::Exit, Layer::Cleanup] {
                    assert_run03_layer(run, layer, Status::Pass);
                }
                assert_run03_layer(run, Layer::Restoration, Status::Blocked);
                assert_run03_layer(run, Layer::Visual, Status::Blocked);
                assert_run03_layer(run, Layer::Ownership, if role == "reference" { Status::NotApplicable } else { Status::Blocked });

                assert_eq!(run.checkpoints.iter().map(|item| item.id.as_str()).collect::<Vec<_>>(),
                    ["00-root", "01-help", "02-root-restored"], "RUN03 checkpoint order changed");
                assert!(run.checkpoints.iter().all(|item| item.status == Status::Pass),
                    "RUN03 checkpoint failed for {role} {kind:?}");
                assert_eq!(run.inputs.iter().map(|item| item.key.as_str()).collect::<Vec<_>>(),
                    ["f1", "escape", "ctrl-c"], "RUN03 key sequence changed");
                assert!(run.inputs.iter().all(|item| item.attempted && item.write_status == "written_and_recorded"),
                    "RUN03 input was not written and recorded for {role} {kind:?}");
                let exit = run.exit.as_ref().unwrap_or_else(|| panic!("RUN03 exit status missing for {role} {kind:?}"));
                assert!(exit.success && exit.code == 0 && !exit.signal_present && exit.stream == "clean_eof",
                    "RUN03 direct child did not exit successfully: {exit:?}");
                assert_eq!(run.restoration.host_terminal_state, "not_observed",
                    "RUN03 must retain the parent-terminal restoration evidence gap");
            }
        }

        let row = &receipt.rows[command_row_index("RUN-03").unwrap()];
        assert_eq!(row.exact_command, "cargo run --release --bin holla");
        assert_eq!(row.status, Status::Blocked,
            "RUN03 aggregate stays blocked while restoration, visual admission, and candidate ownership are unproven");
    }
}

}
