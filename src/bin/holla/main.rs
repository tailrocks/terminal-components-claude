//! Holla: a context-adaptive action launcher for the terminal, as a
//! deterministic preview on the Junie design system. Every external system
//! (mise, git, gh, docker, btm, pg_activity, ssh, the filesystem) is an
//! in-memory simulation; every operator interaction is real. No stack
//! command is ever spawned.

mod app;
#[cfg(test)]
mod app_tests;
#[cfg(test)]
mod app_tests_flows;
#[cfg(test)]
mod app_tests_parity;
#[cfg(test)]
mod app_tests_proofs;
mod clock;
mod domain;
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
                    "holla — this folder, this host, right now (deterministic preview on the Junie design system)\n\n\
                     USAGE: holla [--scenario NAME] [--motion full|reduced|paused] [--frame N] [--color truecolor|256|16|none]\n\n\
                     Scenarios: {concept}\n\
                     Parity:    {parity}\n\
                     Motion:    explicit --motion wins; otherwise HOLLA_NO_MOTION=1 selects reduced motion\n\
                     Frame:     with --motion paused, the exact fixture tick to render\n\n\
                     Keys: type to search · ↑↓ move · Enter run · Alt+Enter alternatives · Tab preview · Ctrl+↑↓ scope · Ctrl+G activities · F1 key reference · Ctrl+Q quit\n\
                     Preview: every scenario is captured under shots/h_*.png; src/bin/holla/README.md lists what each scenario shows.\n\
                     Everything is simulated in memory; no stack command is ever executed.",
                    concept = Scenario::CONCEPT
                        .iter()
                        .map(|s| s.name())
                        .collect::<Vec<_>>()
                        .join(", "),
                    parity = Scenario::PARITY
                        .iter()
                        .map(|s| s.name())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                std::process::exit(0);
            }
            _ => {}
        }
    }
    let no_motion = std::env::var_os("HOLLA_NO_MOTION").is_some_and(|v| !v.is_empty() && v != "0");
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
    if std::env::var_os("HOLLA_NO_HISTORY").is_some_and(|v| v == "1") {
        // the opt-out: nothing is read, learned or written
        app.world.memory.usage = crate::domain::usage::UsageStore::disabled();
    }
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
