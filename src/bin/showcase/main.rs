//! Junie-inspired Ratatui design-system laboratory.

mod app;
#[cfg(test)]
mod app_tests;
mod data;
mod pages;

use crate::app::{App, PageId};
use junie_tui::core::event::{Input, Outcome};
use junie_tui::theme::{ColorLevel, Theme};

struct Options {
    level: ColorLevel,
    page: Option<PageId>,
}

fn parse_args() -> Options {
    let mut level = ColorLevel::detect();
    let mut page = None;
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
            "--page" | "-p" => {
                let name = args.next().unwrap_or_default();
                page = PageId::from_name(&name);
                if page.is_none() {
                    eprintln!("unknown page {name:?}");
                    std::process::exit(2);
                }
            }
            "-h" | "--help" => {
                println!(
                    "junie-tui — Junie-inspired Ratatui design system laboratory\n\n\
                     USAGE: junie-tui [--color truecolor|256|16|none] [--page NAME]\n\n\
                     Keys: Tab/Shift+Tab focus · arrows move · Enter/Space activate · Esc back · [ ] pages · ? help · q quit"
                );
                std::process::exit(0);
            }
            _ => {}
        }
    }
    Options { level, page }
}

fn main() -> std::io::Result<()> {
    let opts = parse_args();
    let theme = Theme::for_level(opts.level);
    let mut app = App::new(theme);
    if let Some(p) = opts.page {
        app.goto(p);
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
