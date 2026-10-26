use std::ffi::{OsStr, OsString};

use crate::{Motion, Scenario};
use termrock::{ColorLevel, Theme};

pub const HELP: &str = "jackin-preview — Jackin redesigned on the Junie design system (deterministic preview)\n\n\
                     USAGE: jackin-preview [--scenario NAME] [--motion full|reduced|paused] [--frame N] [--color truecolor|256|16|none]\n\n\
                     Scenarios: first-use, returning, accounts-mixed, launch-running, launch-failure, capsule-multi, outro-last, hard-cases\n\
                     Motion:    explicit --motion wins; otherwise JACKIN_NO_MOTION=1 selects reduced motion\n\
                     Frame:     with --motion paused, the exact fixture tick to render (intro, cockpit, outro phases)\n\n\
                     Keys: Tab/Shift+Tab focus · ↑↓ move · Enter launch/activate · Esc back · u Accounts & Usage · s Settings · ? help · q quit\n\
                     Everything is simulated in memory; the real Jackin CLI is never touched.";

#[derive(Debug)]
pub struct Options {
    scenario: Scenario,
    motion: Motion,
    frame: u64,
    level: ColorLevel,
    paper: bool,
}

impl Options {
    pub fn run(self) -> std::io::Result<()> {
        let theme = if self.paper {
            Theme::paper()
        } else {
            Theme::junie()
        };
        crate::run_scenario_with_theme(
            self.scenario,
            self.motion,
            self.frame,
            theme.for_level(self.level),
        )
    }
}

#[derive(Debug)]
pub enum Parsed {
    Run(Options),
    Help,
}

type Error = &'static str;

pub fn parse(
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
            _ => {}
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
