//! TablePro, terminal edition: the core database workbench built on the
//! Junie-inspired design system.

mod app;
#[cfg(test)]
mod app_tests;
mod connections;
mod db;
mod model;
#[cfg(test)]
mod perf_tests;
mod sql;
mod tabs;
mod workbench;

use junie_tui::core::event::{Input, Outcome};
use junie_tui::theme::{ColorLevel, Theme};

use crate::app::App;

struct Options {
    level: ColorLevel,
    connect: Option<String>,
}

fn parse_args() -> Options {
    let mut level = ColorLevel::detect();
    let mut connect = None;
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
            "--connect" => connect = args.next(),
            "-h" | "--help" => {
                println!(
                    "tablepro — TablePro's core workflow as a terminal application\n\n\
                     USAGE: tablepro [--color truecolor|256|16|none] [--connect NAME]\n\n\
                     Keys: Ctrl+O open quickly · Ctrl+T new query · Ctrl+R run · Ctrl+Y history · ? help · q quit"
                );
                std::process::exit(0);
            }
            _ => {}
        }
    }
    Options { level, connect }
}

fn main() -> std::io::Result<()> {
    let opts = parse_args();
    let theme = Theme::for_level(opts.level);
    let mut app = App::new(theme);
    if let Some(name) = opts.connect {
        if let Some(i) = app
            .connections
            .connections
            .iter()
            .position(|c| c.name.eq_ignore_ascii_case(&name))
        {
            app.connect(i);
        } else {
            eprintln!("no connection named {name:?}");
            std::process::exit(2);
        }
    }
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
