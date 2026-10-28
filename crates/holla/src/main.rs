#[expect(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "CLI help and diagnostics intentionally use standard streams before terminal acquisition or after terminal teardown"
)]
fn main() -> std::process::ExitCode {
    let args = std::env::args_os()
        .skip(1)
        .map(std::ffi::OsString::into_string)
        .collect::<Result<Vec<_>, _>>();
    let Ok(args) = args else {
        eprintln!("arguments must be valid Unicode");
        return std::process::ExitCode::from(2);
    };
    let no_motion = std::env::var_os("HOLLA_NO_MOTION");
    let options = match holla_ui::cli::parse(args, no_motion.as_deref()) {
        Ok(holla_ui::cli::ParseOutcome::Help) => {
            println!("{}", holla_ui::cli::HELP);
            return std::process::ExitCode::SUCCESS;
        }
        Ok(holla_ui::cli::ParseOutcome::Run(options)) => options,
        Err(error) => {
            eprintln!("{error}");
            return std::process::ExitCode::from(2);
        }
    };
    let color = options
        .color
        .map_or_else(termrock::ColorLevel::detect, |color| match color {
            holla_ui::cli::ColorChoice::TrueColor => termrock::ColorLevel::TrueColor,
            holla_ui::cli::ColorChoice::Ansi256 => termrock::ColorLevel::Ansi256,
            holla_ui::cli::ColorChoice::Ansi16 => termrock::ColorLevel::Ansi16,
            holla_ui::cli::ColorChoice::Mono => termrock::ColorLevel::Mono,
        });
    let mut source_theme = termrock::Theme::junie();
    let menu_title = source_theme
        .recipes
        .get_mut(termrock::Family::MENU)
        .variant_mut(termrock::Variant::custom("termrock.menu-bar-title"))
        .entry(termrock::Part::TITLE);
    menu_title.base = menu_title.base.remove(termrock::Modifier::BOLD);
    let app = holla_ui::App::for_scenario(options.scenario, options.motion, options.frame)
        .with_source_theme(source_theme.clone());
    let Ok(frame) = u64::try_from(app.fixture_time_ms()) else {
        eprintln!("holla: fixture clock precedes its simulation epoch");
        return std::process::ExitCode::FAILURE;
    };
    let feedback = termrock::FeedbackClock::Simulation {
        initial: termrock::SimulationMoment::from_millis(frame),
    };
    match termrock::run_with_feedback_clock(app, source_theme.downgrade(color), feedback) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("holla: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}
