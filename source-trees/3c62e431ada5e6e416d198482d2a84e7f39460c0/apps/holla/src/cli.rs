//! Pure command-line decoding; terminal startup happens only after success.
use std::ffi::OsStr;

use crate::scenario::{Motion, Scenario};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ColorChoice {
    TrueColor,
    Ansi256,
    Ansi16,
    Mono,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Options {
    pub(crate) color: Option<ColorChoice>,
    pub(crate) scenario: Scenario,
    pub(crate) motion: Motion,
    pub(crate) frame: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ParseOutcome {
    Run(Options),
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CliError {
    Color,
    Scenario,
    Motion,
    Frame,
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Never echo arbitrary argument payloads: callers may accidentally pass
        // sensitive text. The failing option and accepted grammar are sufficient.
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

pub(crate) const HELP: &str = "holla — context-adaptive action launcher (deterministic simulation)\n\n\
USAGE: holla [--scenario NAME] [--motion full|reduced|paused] [--frame N] [--color truecolor|256|16|none]\n\n\
Scenarios: first-use, rust-dirty, monorepo-root, monorepo-child, docker-cleanup,\n\
\x20          disk-cleanup, upgrade-plan, activities-multi, remote-host, launch-failure, hard-cases\n\
Motion:    explicit --motion wins; otherwise HOLLA_NO_MOTION=1 selects reduced motion\n\
Frame:     with --motion paused, the exact fixture tick to render\n\n\
Keys: type to filter · ↑↓ move · Enter run · Ctrl+O actions · Ctrl+P preview · F10 menu · F1 help · q quit\n\
Everything is simulated in memory; mise, git, docker and friends are never executed.";

pub(crate) fn parse(
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
            // Preserve the pinned launcher's permissive unknown-option policy.
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

#[cfg(test)]
mod tests {
    use super::*;

    fn args<'a>(values: &'a [&str]) -> impl Iterator<Item = String> + 'a {
        values.iter().map(|value| (*value).to_owned())
    }

    #[test]
    fn defaults_environment_and_explicit_precedence() {
        for (environment, expected) in [
            (None, Motion::Full),
            (Some(""), Motion::Full),
            (Some("0"), Motion::Full),
            (Some("1"), Motion::Reduced),
        ] {
            assert_eq!(
                parse(args(&[]), environment.map(OsStr::new)),
                Ok(ParseOutcome::Run(Options {
                    color: None,
                    scenario: Scenario::FirstUse,
                    motion: expected,
                    frame: 0
                }))
            );
        }
        assert!(matches!(
            parse(args(&["-m", "full"]), Some(OsStr::new("1"))),
            Ok(ParseOutcome::Run(Options {
                motion: Motion::Full,
                ..
            }))
        ));
    }

    #[test]
    fn every_scenario_and_color_alias_round_trips() {
        for scenario in Scenario::ALL {
            assert!(
                matches!(parse(args(&["-s", scenario.name()]), None), Ok(ParseOutcome::Run(options)) if options.scenario == scenario)
            );
        }
        for (name, expected) in [
            ("truecolor", ColorChoice::TrueColor),
            ("24bit", ColorChoice::TrueColor),
            ("256", ColorChoice::Ansi256),
            ("16", ColorChoice::Ansi16),
            ("none", ColorChoice::Mono),
            ("mono", ColorChoice::Mono),
        ] {
            assert!(
                matches!(parse(args(&["-c", name]), None), Ok(ParseOutcome::Run(options)) if options.color == Some(expected))
            );
        }
    }

    #[test]
    fn sequential_help_and_errors_preserve_preterminal_contract() {
        assert_eq!(
            parse(args(&["--help", "--color", "bad"]), None),
            Ok(ParseOutcome::Help)
        );
        assert_eq!(
            parse(args(&["--color", "bad", "--help"]), None),
            Err(CliError::Color)
        );
        for (flag, error) in [
            ("--color", CliError::Color),
            ("--scenario", CliError::Scenario),
            ("--motion", CliError::Motion),
            ("--frame", CliError::Frame),
        ] {
            assert_eq!(parse(args(&[flag]), None), Err(error));
            assert_eq!(parse(args(&[flag, "secret-payload"]), None), Err(error));
            assert!(!error.to_string().contains("secret-payload"));
        }
        assert!(matches!(
            parse(
                args(&["--unknown", "ignored", "-f", "18446744073709551615"]),
                None
            ),
            Ok(ParseOutcome::Run(Options {
                frame: u64::MAX,
                ..
            }))
        ));
        assert_eq!(
            parse(args(&["-f", "18446744073709551616"]), None),
            Err(CliError::Frame)
        );
    }
}
