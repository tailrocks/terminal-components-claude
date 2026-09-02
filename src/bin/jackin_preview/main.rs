//! Jackin, redesigned: an interactive terminal preview of the complete
//! Jackin operator experience built on the Junie design system. Every
//! external system (Docker, daemons, PTYs, providers, 1Password) is a
//! deterministic in-memory simulation; every operator interaction is real.

mod app;
#[cfg(test)]
mod app_tests;
#[cfg(test)]
mod app_tests_chrome;
mod arbiter;
mod clock;
mod domain;
mod rain;
mod scenario;
mod screens;
mod sim;

use junie_tui::core::event::{Input, Outcome};
use junie_tui::theme::{ColorLevel, Theme};

use crate::app::App;
use crate::scenario::{Motion, Scenario};

struct Options {
    level: ColorLevel,
    scenario: Scenario,
    motion: Motion,
    frame: u64,
}

fn parse_args() -> Options {
    let mut level = ColorLevel::detect();
    let mut scenario = Scenario::FirstUse;
    let mut motion = None;
    let mut frame = 0;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--color" | "-c" => {
                level = match args.next().as_deref() {
                    Some("truecolor") | Some("24bit") => ColorLevel::TrueColor,
                    Some("256") => ColorLevel::Ansi256,
                    Some("16") => ColorLevel::Ansi16,
                    Some("none") | Some("mono") => ColorLevel::Mono,
                    other => {
                        eprintln!("unknown --color value {other:?}; use truecolor|256|16|none");
                        std::process::exit(2);
                    }
                };
            }
            "--scenario" | "-s" => {
                let name = args.next().unwrap_or_default();
                scenario = match Scenario::from_name(&name) {
                    Some(s) => s,
                    None => {
                        let names: Vec<&str> = Scenario::ALL.iter().map(|s| s.name()).collect();
                        eprintln!("unknown scenario {name:?}; use one of {}", names.join(", "));
                        std::process::exit(2);
                    }
                };
            }
            "--motion" | "-m" => {
                let name = args.next().unwrap_or_default();
                motion = match Motion::from_name(&name) {
                    Some(m) => Some(m),
                    None => {
                        eprintln!("unknown motion {name:?}; use full|reduced|paused");
                        std::process::exit(2);
                    }
                };
            }
            "--frame" | "-f" => {
                frame = match args.next().and_then(|v| v.parse().ok()) {
                    Some(n) => n,
                    None => {
                        eprintln!("--frame needs a tick number");
                        std::process::exit(2);
                    }
                };
            }
            "-h" | "--help" => {
                println!(
                    "jackin-preview — Jackin redesigned on the Junie design system (deterministic preview)\n\n\
                     USAGE: jackin-preview [--scenario NAME] [--motion full|reduced|paused] [--frame N] [--color truecolor|256|16|none]\n\n\
                     Scenarios: first-use, returning, accounts-mixed, launch-running, launch-failure, capsule-multi, outro-last, hard-cases\n\
                     Motion:    explicit --motion wins; otherwise JACKIN_NO_MOTION=1 selects reduced motion\n\
                     Frame:     with --motion paused, the exact fixture tick to render (intro, cockpit, outro phases)\n\n\
                     Keys: Tab/Shift+Tab focus · ↑↓ move · Enter launch/activate · Esc back · u Accounts & Usage · s Settings · ? help · q quit\n\
                     Everything is simulated in memory; the real Jackin CLI is never touched."
                );
                std::process::exit(0);
            }
            _ => {}
        }
    }
    let no_motion = std::env::var_os("JACKIN_NO_MOTION").is_some_and(|v| !v.is_empty() && v != "0");
    Options {
        level,
        scenario,
        motion: Motion::resolve(motion, no_motion),
        frame,
    }
}

fn main() -> std::io::Result<()> {
    let opts = parse_args();
    let theme = Theme::for_level(opts.level);
    let mut app = App::for_scenario(opts.scenario, opts.motion, opts.frame, theme);
    // stale key presses from the launching shell must not skip the ritual
    let _ = junie_tui::runtime::drain_pending_input();
    junie_tui::runtime::run(&mut app)
}

impl junie_tui::runtime::Application for App {
    fn handle(&mut self, input: Input) -> Outcome {
        App::handle(self, input)
    }
    fn render(&mut self, frame: &mut ratatui::Frame) {
        App::render(self, frame)
    }
    fn should_quit(&self) -> bool {
        self.quit
    }
    fn tick_interval(&self) -> std::time::Duration {
        App::tick_interval(self)
    }
}
