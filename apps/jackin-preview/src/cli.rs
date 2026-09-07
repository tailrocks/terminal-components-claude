//! CLI policy stays pure; terminal initialization occurs only after parsing succeeds.
use std::ffi::{OsStr, OsString};

use jackin_app::{Motion, Scenario};
use junie_tui::{ColorLevel, Theme};

pub(super) const HELP: &str = "jackin-preview — Jackin redesigned on the Junie design system (deterministic preview)\n\n\
                     USAGE: jackin-preview [--scenario NAME] [--motion full|reduced|paused] [--frame N] [--color truecolor|256|16|none]\n\n\
                     Scenarios: first-use, returning, accounts-mixed, launch-running, launch-failure, capsule-multi, outro-last, hard-cases\n\
                     Motion:    explicit --motion wins; otherwise JACKIN_NO_MOTION=1 selects reduced motion\n\
                     Frame:     with --motion paused, the exact fixture tick to render (intro, cockpit, outro phases)\n\n\
                     Keys: Tab/Shift+Tab focus · ↑↓ move · Enter launch/activate · Esc back · u Accounts & Usage · s Settings · ? help · q quit\n\
                     Everything is simulated in memory; the real Jackin CLI is never touched.";

#[derive(Debug)]
pub(super) struct Options {
    scenario: Scenario,
    motion: Motion,
    frame: u64,
    level: ColorLevel,
    paper: bool,
}

impl Options {
    pub(super) fn run(self) -> std::io::Result<()> {
        let theme = if self.paper {
            Theme::paper()
        } else {
            Theme::junie()
        };
        jackin_app::run_scenario_with_theme(
            self.scenario,
            self.motion,
            self.frame,
            theme.for_level(self.level),
        )
    }
}

#[derive(Debug)]
pub(super) enum Parsed {
    Run(Options),
    Help,
}

// Errors contain policy text only, never supplied values. A malformed CLI
// argument can itself be a pasted credential; reference value echoing is unsafe.
type Error = &'static str;

pub(super) fn parse(
    args: impl IntoIterator<Item = OsString>,
    no_motion: Option<&OsStr>,
    detected_color: ColorLevel,
) -> Result<Parsed, Error> {
    let mut scenario = Scenario::FirstUse;
    let mut motion = None;
    let mut frame = 0;
    let mut level = detected_color;
    let mut paper = false;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.to_str() {
            Some("--color" | "-c") => {
                level = args
                    .next()
                    .as_deref()
                    .and_then(OsStr::to_str)
                    .and_then(parse_color)
                    .ok_or("unknown --color value; use truecolor|256|16|none")?;
            }
            Some("--scenario" | "-s") => {
                scenario = args.next().as_deref().and_then(OsStr::to_str).and_then(Scenario::from_name)
                    .ok_or("unknown scenario; use one of first-use, returning, accounts-mixed, launch-running, launch-failure, capsule-multi, outro-last, hard-cases")?;
            }
            Some("--motion" | "-m") => {
                motion = Some(
                    args.next()
                        .as_deref()
                        .and_then(OsStr::to_str)
                        .and_then(Motion::from_name)
                        .ok_or("unknown motion; use full|reduced|paused")?,
                );
            }
            Some("--frame" | "-f") => {
                frame = args
                    .next()
                    .as_deref()
                    .and_then(OsStr::to_str)
                    .and_then(|value| value.parse().ok())
                    .ok_or("--frame needs a tick number")?;
            }
            Some("--theme") => {
                // Preserve main's additive paper selection, including its
                // existing treatment of unrecognized or missing theme values.
                if args
                    .next()
                    .as_deref()
                    .and_then(OsStr::to_str)
                    .is_some_and(|value| value.eq_ignore_ascii_case("paper"))
                {
                    paper = true;
                }
            }
            Some("--help" | "-h") => return Ok(Parsed::Help),
            _ => {} // Pinned reference deliberately ignores unknown arguments.
        }
    }
    let no_motion = no_motion.is_some_and(|value| !value.is_empty() && value != "0");
    Ok(Parsed::Run(Options {
        scenario,
        motion: Motion::resolve(motion, no_motion),
        frame,
        level,
        paper,
    }))
}

fn parse_color(value: &str) -> Option<ColorLevel> {
    match value.to_ascii_lowercase().as_str() {
        "truecolor" | "24bit" => Some(ColorLevel::TrueColor),
        "256" | "ansi256" => Some(ColorLevel::Ansi256),
        "16" | "ansi16" => Some(ColorLevel::Ansi16),
        "none" | "mono" => Some(ColorLevel::Mono),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options(args: &[&str], env: Option<&str>) -> Result<Options, Error> {
        match parse(
            args.iter().map(OsString::from),
            env.map(OsStr::new),
            ColorLevel::Ansi256,
        )? {
            Parsed::Run(options) => Ok(options),
            Parsed::Help => Err("unexpected help"),
        }
    }

    #[test]
    fn first_use_default_and_environment_values_match_reference() -> Result<(), Error> {
        for (env, expected) in [
            (None, Motion::Full),
            (Some(""), Motion::Full),
            (Some("0"), Motion::Full),
            (Some("1"), Motion::Reduced),
            (Some("false"), Motion::Reduced),
        ] {
            let parsed = options(&[], env)?;
            assert_eq!(parsed.scenario, Scenario::FirstUse);
            assert_eq!(parsed.motion, expected);
            assert_eq!(parsed.frame, 0);
            assert_eq!(parsed.level, ColorLevel::Ansi256);
        }
        Ok(())
    }

    #[test]
    fn explicit_motion_wins_over_every_environment_value() -> Result<(), Error> {
        for env in [None, Some(""), Some("0"), Some("1")] {
            for (value, expected) in [
                ("full", Motion::Full),
                ("reduced", Motion::Reduced),
                ("paused", Motion::Paused),
            ] {
                assert_eq!(options(&["--motion", value], env)?.motion, expected);
            }
        }
        Ok(())
    }

    #[test]
    fn aliases_last_valid_options_and_main_additions_survive() -> Result<(), Error> {
        let parsed = options(
            &[
                "--scenario",
                "first-use",
                "-s",
                "returning",
                "--motion",
                "reduced",
                "-m",
                "paused",
                "--frame",
                "1",
                "-f",
                "18446744073709551615",
                "--color",
                "none",
                "-c",
                "ANSI16",
                "--theme",
                "PaPeR",
            ],
            Some("1"),
        )?;
        assert_eq!(parsed.scenario, Scenario::Returning);
        assert_eq!(parsed.motion, Motion::Paused);
        assert_eq!(parsed.frame, u64::MAX);
        assert_eq!(parsed.level, ColorLevel::Ansi16);
        assert!(parsed.paper);
        for scenario in Scenario::ALL {
            assert_eq!(options(&["-s", scenario.name()], None)?.scenario, scenario);
        }
        for (value, level) in [
            ("truecolor", ColorLevel::TrueColor),
            ("24bit", ColorLevel::TrueColor),
            ("256", ColorLevel::Ansi256),
            ("ansi256", ColorLevel::Ansi256),
            ("ANSI256", ColorLevel::Ansi256),
            ("16", ColorLevel::Ansi16),
            ("ansi16", ColorLevel::Ansi16),
            ("none", ColorLevel::Mono),
            ("mono", ColorLevel::Mono),
        ] {
            assert_eq!(options(&["-c", value], None)?.level, level);
        }
        assert!(options(&["-f", "18446744073709551616"], None).is_err());
        assert!(options(&["-f", "-1"], None).is_err());
        Ok(())
    }
}
