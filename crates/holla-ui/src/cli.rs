use std::ffi::OsStr;

use crate::scenario::{Motion, Scenario};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorChoice {
    TrueColor,
    Ansi256,
    Ansi16,
    Mono,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Options {
    pub color: Option<ColorChoice>,
    pub scenario: Scenario,
    pub motion: Motion,
    pub frame: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseOutcome {
    Run(Options),
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliError {
    Color,
    Scenario,
    Motion,
    Frame,
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Color => f.write_str("unknown --color value; use truecolor|256|16|none"),
            Self::Scenario => write!(
                f,
                "unknown scenario; use one of {}",
                Scenario::ALL.map(Scenario::name).join(", ")
            ),
            Self::Motion => f.write_str("unknown motion; use full|reduced|paused"),
            Self::Frame => f.write_str("--frame needs a tick number"),
        }
    }
}
impl std::error::Error for CliError {}

pub const HELP: &str = "holla — context-adaptive action launcher (deterministic simulation)\n\n\
USAGE: holla [--scenario NAME] [--motion full|reduced|paused] [--frame N] [--color truecolor|256|16|none]\n\n\
Scenarios: first-use, rust-dirty, monorepo-root, monorepo-child, docker-cleanup,\n\
\x20          disk-cleanup, upgrade-plan, activities-multi, remote-host, launch-failure, hard-cases,\n\
\x20          parity-discovery, parity-history, parity-files, parity-browser, parity-git-current,\n\
\x20          parity-git-batch, parity-task-sources, parity-cargo, parity-docker, parity-brew-services,\n\
\x20          parity-gradle, parity-idea, parity-upgrade-managers, parity-executor, parity-task-input,\n\
\x20          parity-custom-actions, parity-disk-scan, parity-disk-navigation, parity-insights,\n\
\x20          parity-delete-safety, parity-cleanup-results, parity-platforms, parity-platforms-linux,\n\
\x20          parity-activity-stop\n\
Motion:    explicit --motion wins; otherwise HOLLA_NO_MOTION=1 selects reduced motion\n\
Frame:     with --motion paused, the exact fixture tick to render\n\n\
Keys: type to filter · ↑↓ move · Enter run · Ctrl+O actions · Ctrl+P preview · F10 menu · F1 help · q quit\n\
Everything is simulated in memory; mise, git, docker and friends are never executed.";

pub fn parse(
    args: impl IntoIterator<Item = String>,
    no_motion_env: Option<&OsStr>,
) -> Result<ParseOutcome, CliError> {
    let mut color = None;
    let mut scenario = Scenario::FirstUse;
    let mut motion = None;
    let mut frame = 0;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--color" | "-c" => {
                color = Some(match args.next().as_deref() {
                    Some("truecolor" | "24bit") => ColorChoice::TrueColor,
                    Some("256") => ColorChoice::Ansi256,
                    Some("16") => ColorChoice::Ansi16,
                    Some("none" | "mono") => ColorChoice::Mono,
                    _ => return Err(CliError::Color),
                });
            }
            "--scenario" | "-s" => {
                scenario = args
                    .next()
                    .as_deref()
                    .and_then(Scenario::from_name)
                    .ok_or(CliError::Scenario)?;
            }
            "--motion" | "-m" => {
                motion = Some(
                    args.next()
                        .as_deref()
                        .and_then(Motion::from_name)
                        .ok_or(CliError::Motion)?,
                );
            }
            "--frame" | "-f" => {
                frame = args
                    .next()
                    .and_then(|value| value.parse().ok())
                    .ok_or(CliError::Frame)?;
            }
            "--help" | "-h" => return Ok(ParseOutcome::Help),

            _ => {}
        }
    }
    Ok(ParseOutcome::Run(Options {
        color,
        scenario,
        motion: Motion::resolve(
            motion,
            no_motion_env.is_some_and(|value| !value.is_empty() && value != "0"),
        ),
        frame,
    }))
}
