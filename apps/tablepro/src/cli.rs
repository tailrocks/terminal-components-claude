//! Command-line decisions are resolved before acquiring a terminal session.

use std::io::{self, Write};

use junie_tui::{ColorLevel, Theme};

const HELP: &str = "tablepro — TablePro's core workflow as a terminal application\n\n\
USAGE: tablepro [--color truecolor|256|16|none] [--connect NAME] [--theme junie|paper]\n\n\
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
    },
}

/// Resolve command-line options and start the interactive application, or print help.
///
/// # Errors
/// Returns invalid-input errors for invalid option values or unknown connections,
/// and I/O errors for output or terminal failures. Diagnostics omit argument values.
pub fn run() -> io::Result<()> {
    match parse_args(std::env::args().skip(1), ColorLevel::detect())? {
        Command::Help => io::stdout().lock().write_all(HELP.as_bytes()),
        Command::Run {
            theme,
            level,
            connect,
        } => crate::run_with(theme.resolve(level), connect.as_deref()),
    }
}

fn invalid_arg(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}

fn parse_args(args: impl IntoIterator<Item = String>, detected: ColorLevel) -> io::Result<Command> {
    let mut theme = ThemeChoice::Junie;
    let mut level = detected;
    let mut connect = None;
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
            // The pinned CLI accepts an absent connection value and ignores unknown
            // arguments. Preserve that behavior independently of parser structure.
            "--connect" => connect = args.next(),
            "-h" | "--help" => return Ok(Command::Help),
            _ => {}
        }
    }
    Ok(Command::Run {
        theme,
        level,
        connect,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_aliases_override_detection_independent_of_theme_order() -> io::Result<()> {
        for (name, level) in [
            ("truecolor", ColorLevel::TrueColor),
            ("24bit", ColorLevel::TrueColor),
            ("256", ColorLevel::Ansi256),
            ("ansi256", ColorLevel::Ansi256),
            ("16", ColorLevel::Ansi16),
            ("ansi16", ColorLevel::Ansi16),
            ("none", ColorLevel::Mono),
            ("mono", ColorLevel::Mono),
        ] {
            for flag in ["--color", "-c"] {
                for args in [
                    [flag, name, "--theme", "paper"],
                    ["--theme", "paper", flag, name],
                ] {
                    let command = parse_args(args.map(str::to_owned), ColorLevel::Mono)?;
                    let Command::Run {
                        theme,
                        level: actual,
                        ..
                    } = command
                    else {
                        return Err(io::Error::other("expected run command"));
                    };
                    assert_eq!(theme.resolve(actual), Theme::paper().for_level(level));
                }
            }
        }
        Ok(())
    }

    #[test]
    fn detection_and_permissive_reference_options_survive() -> io::Result<()> {
        let Command::Run {
            theme,
            level,
            connect,
        } = parse_args(
            ["--unknown", "value", "--connect"].map(str::to_owned),
            ColorLevel::Ansi16,
        )?
        else {
            return Err(io::Error::other("expected run command"));
        };
        assert_eq!(
            theme.resolve(level),
            Theme::junie().for_level(ColorLevel::Ansi16)
        );
        assert!(connect.is_none());
        Ok(())
    }
}
