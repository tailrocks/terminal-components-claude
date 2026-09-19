//! Command-line decoding for theme, page, and paused-frame seeking.
//! Terminal acquisition happens only after this succeeds.

use crate::app::{Motion, PageId};
use junie_tui::{ColorLevel, Theme};

#[derive(Debug, Clone)]
pub(crate) struct Options {
    pub(crate) theme: Theme,
    pub(crate) page: PageId,
    pub(crate) motion: Motion,
    pub(crate) frame: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CliError {
    Motion,
    Frame,
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Motion => f.write_str("unknown motion; use full|reduced|paused"),
            Self::Frame => f.write_str("--frame needs a tick number"),
        }
    }
}

impl std::error::Error for CliError {}

pub(crate) fn parse(args: impl IntoIterator<Item = String>) -> Result<Options, CliError> {
    let mut theme = Theme::junie();
    let mut page = PageId::Overview;
    let mut motion = Motion::Full;
    let mut frame = 0;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--theme" => {
                if let Some(value) = args.next() {
                    theme = if value.eq_ignore_ascii_case("paper") {
                        Theme::paper()
                    } else {
                        Theme::junie()
                    };
                }
            }
            "--color" | "-c" => {
                if let Some(value) = args.next() {
                    let level = match value.to_ascii_lowercase().as_str() {
                        "truecolor" | "24bit" => Some(ColorLevel::TrueColor),
                        "256" | "ansi256" => Some(ColorLevel::Ansi256),
                        "16" | "ansi16" => Some(ColorLevel::Ansi16),
                        "none" | "mono" => Some(ColorLevel::Mono),
                        _ => None,
                    };
                    if let Some(level) = level {
                        theme = theme.downgrade(level);
                    }
                }
            }
            "--page" | "-p" => {
                if let Some(value) = args.next()
                    && let Some(selected) = PageId::from_name(&value)
                {
                    page = selected;
                }
            }
            "--motion" | "-m" => {
                let value = args.next().ok_or(CliError::Motion)?;
                motion = match value.as_str() {
                    "full" | "reduced" => Motion::Full,
                    "paused" => Motion::Paused,
                    _ => return Err(CliError::Motion),
                };
            }
            "--frame" | "-f" => {
                frame = args
                    .next()
                    .ok_or(CliError::Frame)?
                    .parse()
                    .map_err(|_| CliError::Frame)?;
            }
            _ => {}
        }
    }
    Ok(Options {
        theme,
        page,
        motion,
        frame,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn defaults_keep_live_overview() {
        assert!(matches!(
            parse(args(&[])),
            Ok(Options {
                page: PageId::Overview,
                motion: Motion::Full,
                frame: 0,
                ..
            })
        ));
    }

    #[test]
    fn paused_frame_seeks_and_selects_progress() {
        assert!(matches!(
            parse(args(&[
                "--page", "progress", "--motion", "paused", "--frame", "80",
            ])),
            Ok(Options {
                page: PageId::Progress,
                motion: Motion::Paused,
                frame: 80,
                ..
            })
        ));
    }

    #[test]
    fn short_flags_and_reduced_motion_match_tag() {
        assert!(matches!(
            parse(args(&["-p", "scrolling", "-m", "reduced", "-f", "1600"])),
            Ok(Options {
                page: PageId::Scrolling,
                motion: Motion::Full,
                frame: 1600,
                ..
            })
        ));
        assert!(matches!(
            parse(args(&["--motion", "paused", "--frame", "0"])),
            Ok(Options {
                motion: Motion::Paused,
                frame: 0,
                ..
            })
        ));
    }

    #[test]
    fn invalid_motion_and_frame_fail_before_terminal() {
        assert!(matches!(parse(args(&["--motion"])), Err(CliError::Motion)));
        assert!(matches!(
            parse(args(&["--motion", "wiggly"])),
            Err(CliError::Motion)
        ));
        assert!(matches!(parse(args(&["--frame"])), Err(CliError::Frame)));
        assert!(matches!(parse(args(&["-f", "nope"])), Err(CliError::Frame)));
        let error = CliError::Motion;
        assert!(!error.to_string().contains("wiggly"));
        assert!(!CliError::Frame.to_string().contains("nope"));
    }
}
