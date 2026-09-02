//! Junie-inspired Ratatui design-system laboratory.

mod app;
#[cfg(test)]
mod app_tests;
mod core;
mod data;
mod pages;
mod theme;
mod ui;
mod widgets;

use std::io::stdout;
use std::time::{Duration, Instant};

use ratatui::crossterm::event::{
    self, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
};
use ratatui::crossterm::execute;

use crate::app::{App, PageId};
use crate::core::event::{Input, Outcome};
use crate::theme::{ColorLevel, Theme};

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
    let mut terminal = ratatui::init();
    execute!(stdout(), EnableMouseCapture, EnableBracketedPaste)?;
    let result = run(&mut terminal, &mut app);
    let _ = execute!(stdout(), DisableMouseCapture, DisableBracketedPaste);
    ratatui::restore();
    result
}

fn run(terminal: &mut ratatui::DefaultTerminal, app: &mut App) -> std::io::Result<()> {
    let mut dirty = true;
    let mut last_tick = Instant::now();
    loop {
        if dirty {
            terminal.draw(|f| app.render(f))?;
            dirty = false;
        }
        let interval = app.tick_interval();
        let wait = interval.saturating_sub(last_tick.elapsed());
        if event::poll(wait)? {
            // drain everything pending to coalesce mouse-move floods
            loop {
                let ev = event::read()?;
                if let Some(input) = Input::from_crossterm(ev)
                    && app.handle(input) == Outcome::Changed
                {
                    dirty = true;
                }
                if app.quit {
                    return Ok(());
                }
                if !event::poll(Duration::ZERO)? {
                    break;
                }
            }
        }
        if last_tick.elapsed() >= interval {
            last_tick = Instant::now();
            if app.handle(Input::Tick) == Outcome::Changed {
                dirty = true;
            }
        }
    }
}
