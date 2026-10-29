use std::io::{self, Write};

use termrock::{ColorLevel, Theme};

use crate::app::Motion;

const HELP: &str = "tablepro — TablePro's core workflow as a terminal application\n\n\
USAGE: tablepro [--color truecolor|256|16|none] [--connect NAME] [--theme junie|paper]\n\
         [--motion full|reduced|paused] [--frame N]\n\n\
Motion:    explicit --motion wins; otherwise TABLEPRO_NO_MOTION=1 selects reduced motion\n\
Frame:     with --motion paused, the exact tick to render\n\n\
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
        } => crate::app::run_session(theme.resolve(level), connect.as_deref(), motion, frame),
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
                    let command = parse_args(args.map(str::to_owned), ColorLevel::Mono, false)?;
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
            motion,
            frame,
        } = parse_args(
            ["--unknown", "value", "--connect"].map(str::to_owned),
            ColorLevel::Ansi16,
            false,
        )?
        else {
            return Err(io::Error::other("expected run command"));
        };
        assert_eq!(
            theme.resolve(level),
            Theme::junie().for_level(ColorLevel::Ansi16)
        );
        assert!(connect.is_none());
        assert_eq!((motion, frame), (Motion::Full, 0));
        Ok(())
    }

    #[test]
    fn motion_frame_flags_and_schema_rows_match_the_baseline() -> io::Result<()> {
        use crate::app::{Surface, TableProApp};
        use termrock::Theme;
        use termrock_test_support::Harness;

        let Command::Run {
            motion: paused,
            frame: paused_frame,
            ..
        } = parse_args(
            ["--motion", "paused", "--frame", "40"].map(str::to_owned),
            ColorLevel::TrueColor,
            true,
        )?
        else {
            return Err(io::Error::other("expected run command"));
        };
        assert_eq!((paused, paused_frame), (Motion::Paused, 40));
        let Command::Run { motion, frame, .. } = parse_args(
            ["-m", "reduced", "-f", "3"].map(str::to_owned),
            ColorLevel::TrueColor,
            false,
        )?
        else {
            return Err(io::Error::other("expected run command"));
        };
        assert_eq!((motion, frame), (Motion::Reduced, 3));
        let Command::Run { motion, frame, .. } = parse_args(
            ["--motion", "full"].map(str::to_owned),
            ColorLevel::TrueColor,
            true,
        )?
        else {
            return Err(io::Error::other("expected run command"));
        };
        assert_eq!((motion, frame), (Motion::Full, 0));
        let Command::Run { motion, .. } =
            parse_args(std::iter::empty::<String>(), ColorLevel::TrueColor, true)?
        else {
            return Err(io::Error::other("expected run command"));
        };
        assert_eq!(motion, Motion::Reduced);
        for args in [
            &["--motion", "slow"][..],
            &["--motion"],
            &["--frame", "-1"],
            &["--frame"],
        ] {
            let Err(error) = parse_args(
                args.iter().map(|arg| (*arg).to_owned()),
                ColorLevel::TrueColor,
                false,
            ) else {
                return Err(io::Error::other("invalid motion or frame was accepted"));
            };
            assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
            assert!(!error.to_string().contains("slow"));
            assert!(!error.to_string().contains("-1"));
        }

        let mut app = TableProApp::default();
        app.set_surface(Surface::WorkbenchDefault);
        app.set_motion(Motion::Paused, paused_frame);
        let debug = format!("{app:?}");
        assert!(debug.contains("motion: Paused"), "{debug}");
        assert!(debug.contains("tick: 40"), "{debug}");
        assert!(matches!(
            crate::app::feedback_clock(Motion::Paused),
            termrock::FeedbackClock::Simulation { .. }
        ));
        let harness = Harness::new(app, Theme::junie(), 120, 40);
        let text = harness.text();
        assert!(
            text.contains("S public"),
            "schema row must use the baseline glyph"
        );
        assert!(!text.contains("▾ public"));
        let schema_rows: Vec<_> = text
            .lines()
            .filter(|line| {
                line.contains("S public")
                    || line.contains("S analytics")
                    || line.contains("S audit")
            })
            .collect();
        assert!(schema_rows.iter().any(|line| line.contains("S public")));
        let barred = schema_rows.iter().filter(|line| line.contains('▎')).count();
        assert!(
            barred <= 1,
            "one focus bar across schema rows, got {barred}: {schema_rows:?}"
        );
        Ok(())
    }
}
