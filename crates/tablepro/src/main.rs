//! `TablePro` binary entry point.

fn main() -> std::process::ExitCode {
    use std::io::Write as _;
    match tablepro_ui::run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            let _ = writeln!(std::io::stderr().lock(), "{error}");
            if error.kind() == std::io::ErrorKind::InvalidInput {
                std::process::ExitCode::from(2)
            } else {
                std::process::ExitCode::FAILURE
            }
        }
    }
}
