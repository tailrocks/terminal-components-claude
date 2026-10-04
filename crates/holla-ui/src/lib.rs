//! Holla's deterministic application model and simulation boundary.
#![forbid(unsafe_code)]

pub mod app;
pub mod cli;
pub mod dispatch;
pub mod screens;

pub mod domain {
    pub use holla_catalog::ranking;
    pub use holla_domain::*;
    pub use holla_plan::{activity, plan};
    pub use holla_sim::fixtures;
}
pub mod catalog {
    pub use holla_catalog::*;
}
pub mod plan {
    pub use holla_plan::*;
}
pub mod sim {
    pub use holla_sim::*;
}
pub mod scenario {
    pub use holla_domain::scenario::*;
}
pub mod clock {
    pub use holla_domain::clock::*;
}

pub use app::App;
pub use scenario::{Motion, Scenario};

/// Run the launcher, decoding help and argument errors before opening a terminal.
/// Returns status two for invalid arguments and status one for terminal failures.
#[expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "CLI help and diagnostics intentionally use standard streams before terminal acquisition or after terminal teardown"
)]
pub fn run() -> std::process::ExitCode {
    let args = std::env::args_os()
        .skip(1)
        .map(std::ffi::OsString::into_string)
        .collect::<Result<Vec<_>, _>>();
    let Ok(args) = args else {
        eprintln!("arguments must be valid Unicode");
        return std::process::ExitCode::from(2);
    };
    let no_motion = std::env::var_os("HOLLA_NO_MOTION");
    let options = match cli::parse(args, no_motion.as_deref()) {
        Ok(cli::ParseOutcome::Help) => {
            println!("{}", cli::HELP);
            return std::process::ExitCode::SUCCESS;
        }
        Ok(cli::ParseOutcome::Run(options)) => options,
        Err(error) => {
            eprintln!("{error}");
            return std::process::ExitCode::from(2);
        }
    };
    let color = options
        .color
        .map_or_else(termrock::ColorLevel::detect, |color| match color {
            cli::ColorChoice::TrueColor => termrock::ColorLevel::TrueColor,
            cli::ColorChoice::Ansi256 => termrock::ColorLevel::Ansi256,
            cli::ColorChoice::Ansi16 => termrock::ColorLevel::Ansi16,
            cli::ColorChoice::Mono => termrock::ColorLevel::Mono,
        });
    let app = App::for_scenario(options.scenario, options.motion, options.frame);
    let Ok(frame) = u64::try_from(app.fixture_time_ms()) else {
        eprintln!("holla: fixture clock precedes its simulation epoch");
        return std::process::ExitCode::FAILURE;
    };
    let feedback = termrock::FeedbackClock::Simulation {
        initial: termrock::SimulationMoment::from_millis(frame),
    };
    match termrock::run_with_feedback_clock(
        app,
        termrock::Theme::junie().downgrade(color),
        feedback,
    ) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("holla: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
