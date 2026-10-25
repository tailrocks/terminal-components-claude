use std::io::{self, Write};
use std::time::Duration;

use termrock::{ColorLevel, Theme};

use crate::app::Motion;

const HELP: &str = "tablepro — TablePro's core workflow as a terminal application\n\n\
USAGE: tablepro [--color truecolor|256|16|none] [--connect NAME] [--theme junie|paper]\n\
         [--motion full|reduced|paused] [--frame N] [--query-duration-ms N]\n\n\
Motion:    explicit --motion wins; otherwise TABLEPRO_NO_MOTION=1 selects reduced motion\n\
Frame:     with --motion paused, the exact tick to render\n\n\
Query time: fixed reported duration for deterministic replay; absent means live timing\n\
Keys: Ctrl+O open quickly · Ctrl+T new query · Ctrl+R run · Ctrl+Y history · ? help · q quit\n";

#[derive(Clone, Copy)]
enum ThemeChoice {
    Junie,
    Paper,
}

impl ThemeChoice {
    fn resolve(self, level: ColorLevel) -> Theme {
        match self {
            Self::Junie => Theme::junie(),
            Self::Paper => Theme::paper(),
        }
        .for_level(level)
    }
}

enum Command {
    Help,
    Run {
        theme: ThemeChoice,
        level: ColorLevel,
        connect: Option<String>,
        motion: Motion,
        frame: u64,
        query_duration: Option<Duration>,
    },
}

pub fn run() -> io::Result<()> {
    let no_motion = std::env::var_os("TABLEPRO_NO_MOTION")
        .is_some_and(|value| !value.is_empty() && value != "0");
    match parse_args(std::env::args().skip(1), ColorLevel::detect(), no_motion)? {
        Command::Help => io::stdout().lock().write_all(HELP.as_bytes()),
        Command::Run {
            theme,
            level,
            connect,
            motion,
            frame,
            query_duration,
        } => {
            let theme = theme.resolve(level);
            if let Some(duration) = query_duration {
                crate::app::run_session_with_query_duration(
                    theme,
                    connect.as_deref(),
                    motion,
                    frame,
                    duration,
                )
            } else {
                crate::app::run_session(theme, connect.as_deref(), motion, frame)
            }
        }
    }
}

fn invalid_arg(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}

fn parse_args(
    args: impl IntoIterator<Item = String>,
    detected: ColorLevel,
    no_motion: bool,
) -> io::Result<Command> {
    let mut theme = ThemeChoice::Junie;
    let mut level = detected;
    let mut connect = None;
    let mut motion = None;
    let mut frame = 0u64;
    let mut query_duration = None;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--theme" => {
                let value = args
                    .next()
                    .ok_or_else(|| invalid_arg("--theme requires a value"))?;
                theme = match value.to_ascii_lowercase().as_str() {
                    "junie" => ThemeChoice::Junie,
                    "paper" => ThemeChoice::Paper,
                    _ => return Err(invalid_arg("--theme must be junie or paper")),
                };
            }
            "--color" | "-c" => {
                let value = args
                    .next()
                    .ok_or_else(|| invalid_arg("--color requires a value"))?;
                level = match value.to_ascii_lowercase().as_str() {
                    "truecolor" | "24bit" => ColorLevel::TrueColor,
                    "256" | "ansi256" => ColorLevel::Ansi256,
                    "16" | "ansi16" => ColorLevel::Ansi16,
                    "none" | "mono" => ColorLevel::Mono,
                    _ => return Err(invalid_arg("--color must be truecolor, 256, 16, or none")),
                };
            }

            "--connect" => connect = args.next(),
            "--motion" | "-m" => {
                let value = args
                    .next()
                    .ok_or_else(|| invalid_arg("--motion requires a value"))?;
                motion = Some(match value.to_ascii_lowercase().as_str() {
                    "full" => Motion::Full,
                    "reduced" => Motion::Reduced,
                    "paused" => Motion::Paused,
                    _ => return Err(invalid_arg("--motion must be full, reduced, or paused")),
                });
            }
            "--frame" | "-f" => {
                let value = args
                    .next()
                    .ok_or_else(|| invalid_arg("--frame requires a value"))?;
                frame = value
                    .parse()
                    .map_err(|_| invalid_arg("--frame must be a non-negative integer"))?;
            }
            "--query-duration-ms" => {
                let value = args
                    .next()
                    .ok_or_else(|| invalid_arg("--query-duration-ms requires a value"))?;
                let milliseconds = value.parse::<u64>().map_err(|_| {
                    invalid_arg("--query-duration-ms must be a non-negative integer")
                })?;
                query_duration = Some(Duration::from_millis(milliseconds));
            }
            "-h" | "--help" => return Ok(Command::Help),
            _ => {}
        }
    }
    Ok(Command::Run {
        theme,
        level,
        connect,
        motion: motion.unwrap_or(if no_motion {
            Motion::Reduced
        } else {
            Motion::Full
        }),
        frame,
        query_duration,
    })
}
