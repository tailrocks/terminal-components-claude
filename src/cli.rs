//! Command-line options every application shares: colour level, motion and
//! the paused frame. An application flattens [`CommonArgs`] into its own
//! `clap` parser and adds only its own options:
//!
//! ```ignore
//! #[derive(clap::Parser)]
//! struct Cli {
//!     #[command(flatten)]
//!     common: junie_tui::cli::CommonArgs,
//!     #[arg(long)]
//!     connect: Option<String>,
//! }
//! let cli: Cli = junie_tui::cli::parse(after_help);
//! let theme = cli.common.theme();
//! let motion = cli.common.motion("TABLEPRO_NO_MOTION");
//! ```
//!
//! `--help` prints usage and exits 0; an unknown option or value prints an
//! error and exits non-zero (clap's contract).

use clap::{Parser, ValueEnum};

use crate::motion::Motion;
use crate::theme::{ColorLevel, Theme};

/// `--color` values. `24bit` and `mono` are aliases.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum ColorArg {
    #[value(name = "truecolor", alias = "24bit")]
    TrueColor,
    #[value(name = "256")]
    Ansi256,
    #[value(name = "16")]
    Ansi16,
    #[value(name = "none", alias = "mono")]
    Mono,
}

impl From<ColorArg> for ColorLevel {
    fn from(value: ColorArg) -> Self {
        match value {
            ColorArg::TrueColor => Self::TrueColor,
            ColorArg::Ansi256 => Self::Ansi256,
            ColorArg::Ansi16 => Self::Ansi16,
            ColorArg::Mono => Self::Mono,
        }
    }
}

/// The options every application accepts.
#[derive(Clone, Debug, Default, PartialEq, Eq, clap::Args)]
pub struct CommonArgs {
    /// Colour level (default: detected from the terminal; NO_COLOR selects none)
    #[arg(short = 'c', long, value_enum, value_name = "LEVEL")]
    pub color: Option<ColorArg>,
    /// Motion: full, reduced (decorative motion stays still) or paused (ticks never advance)
    #[arg(short = 'm', long, value_enum, value_name = "MOTION")]
    pub motion: Option<Motion>,
    /// With --motion paused, the tick to render
    #[arg(short = 'f', long, default_value_t = 0, value_name = "N")]
    pub frame: u64,
}

impl CommonArgs {
    /// The colour level: `--color`, else [`ColorLevel::detect`].
    pub fn level(&self) -> ColorLevel {
        self.color
            .map(ColorLevel::from)
            .unwrap_or_else(ColorLevel::detect)
    }

    /// The theme for [`level`](Self::level).
    pub fn theme(&self) -> Theme {
        Theme::for_level(self.level())
    }

    /// The motion level: `--motion`, else reduced when the application's
    /// `env_var` (`<APP>_NO_MOTION`) is set, else full
    /// ([`Motion::from_cli_or_env`]).
    pub fn motion(&self, env_var: &str) -> Motion {
        Motion::from_cli_or_env(self.motion, env_var)
    }
}

/// The `Motion:` line of an application's after-help text, naming its
/// `<APP>_NO_MOTION` variable.
pub fn motion_help(env_var: &str) -> String {
    format!(
        "Motion:    explicit --motion wins; otherwise {env_var}=1 selects reduced motion; \
         --motion paused pins tick-derived surfaces at --frame N for capture"
    )
}

/// Parse the process arguments into `C` with `after_help` appended to its
/// help text. `--help` prints and exits 0; an invalid command line prints
/// the error and exits non-zero.
pub fn parse<C: Parser>(after_help: String) -> C {
    let matches = C::command().after_help(after_help).get_matches();
    C::from_arg_matches(&matches).unwrap_or_else(|error| error.exit())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::error::ErrorKind;

    #[derive(Debug, Parser)]
    #[command(name = "app")]
    struct Cli {
        #[command(flatten)]
        common: CommonArgs,
        #[arg(long)]
        extra: Option<String>,
    }

    #[test]
    fn common_options_parse_long_and_short() {
        let cli = Cli::try_parse_from([
            "app", "--color", "16", "--motion", "reduced", "--frame", "9", "--extra", "x",
        ])
        .unwrap();
        assert_eq!(cli.common.color, Some(ColorArg::Ansi16));
        assert_eq!(cli.common.motion, Some(Motion::Reduced));
        assert_eq!(cli.common.frame, 9);
        assert_eq!(cli.extra.as_deref(), Some("x"));
        let short = Cli::try_parse_from(["app", "-c", "24bit", "-m", "paused", "-f", "3"]).unwrap();
        assert_eq!(short.common.color, Some(ColorArg::TrueColor));
        assert_eq!(short.common.motion, Some(Motion::Paused));
        assert_eq!(short.common.frame, 3);
        assert_eq!(short.common.level(), ColorLevel::TrueColor);
        let none = Cli::try_parse_from(["app"]).unwrap();
        assert_eq!(none.common, CommonArgs::default());
    }

    #[test]
    fn every_colour_and_motion_value_is_accepted() {
        for color in ["truecolor", "24bit", "256", "16", "none", "mono"] {
            assert!(
                Cli::try_parse_from(["app", "--color", color]).is_ok(),
                "{color}"
            );
        }
        for motion in ["full", "reduced", "paused"] {
            assert!(
                Cli::try_parse_from(["app", "--motion", motion]).is_ok(),
                "{motion}"
            );
        }
        let mono = Cli::try_parse_from(["app", "--color", "mono"]).unwrap();
        assert_eq!(mono.common.level(), ColorLevel::Mono);
    }

    #[test]
    fn invalid_command_lines_are_errors() {
        let kind = |args: &[&str]| Cli::try_parse_from(args).unwrap_err().kind();
        assert_eq!(kind(&["app", "--color", "bogus"]), ErrorKind::InvalidValue);
        assert_eq!(kind(&["app", "--motion", "bogus"]), ErrorKind::InvalidValue);
        assert_eq!(kind(&["app", "--frame"]), ErrorKind::InvalidValue);
        assert_eq!(kind(&["app", "--bogus"]), ErrorKind::UnknownArgument);
        assert_eq!(kind(&["app", "--help"]), ErrorKind::DisplayHelp);
    }

    #[test]
    fn explicit_motion_wins_and_help_names_the_variable() {
        let cli = Cli::try_parse_from(["app", "--motion", "paused"]).unwrap();
        assert_eq!(
            cli.common
                .motion("JUNIE_TUI_TEST_MOTION_VARIABLE_THAT_IS_NEVER_SET"),
            Motion::Paused
        );
        let none = Cli::try_parse_from(["app"]).unwrap();
        assert_eq!(
            none.common
                .motion("JUNIE_TUI_TEST_MOTION_VARIABLE_THAT_IS_NEVER_SET"),
            Motion::Full
        );
        assert!(motion_help("APP_NO_MOTION").contains("APP_NO_MOTION=1"));
    }
}
