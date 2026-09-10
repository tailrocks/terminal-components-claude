//! Command-line entry point for the deterministic Jackin Preview shell.
mod cli;

use std::io::Write as _;

fn main() -> std::io::Result<()> {
    let no_motion = std::env::var_os("JACKIN_NO_MOTION");
    match cli::parse(
        std::env::args_os().skip(1),
        no_motion.as_deref(),
        junie_tui::ColorLevel::detect(),
    ) {
        Ok(cli::Parsed::Run(options)) => options.run(),
        Ok(cli::Parsed::Help) => {
            writeln!(std::io::stdout().lock(), "{}", cli::HELP)?;
            Ok(())
        }
        Err(message) => {
            let _ = writeln!(std::io::stderr().lock(), "{message}");
            std::process::exit(2);
        }
    }
}
