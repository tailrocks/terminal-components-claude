//! Command-line entry point for the deterministic Jackin Preview shell.
use std::io::Write as _;

fn main() -> std::io::Result<()> {
    let no_motion = std::env::var_os("JACKIN_NO_MOTION");
    match jackin_preview_app::cli::parse(
        std::env::args_os().skip(1),
        no_motion.as_deref(),
        termrock::ColorLevel::detect(),
    ) {
        Ok(jackin_preview_app::cli::Parsed::Run(options)) => options.run(),
        Ok(jackin_preview_app::cli::Parsed::Help) => {
            writeln!(
                std::io::stdout().lock(),
                "{}",
                jackin_preview_app::cli::HELP
            )?;
            Ok(())
        }
        Err(message) => {
            let _ = writeln!(std::io::stderr().lock(), "{message}");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_jackin_cli_help() {
        let parsed = jackin_preview_app::cli::parse(
            vec![std::ffi::OsString::from("--help")],
            None,
            termrock::ColorLevel::detect(),
        );
        assert!(matches!(parsed, Ok(jackin_preview_app::cli::Parsed::Help)));
    }
}
