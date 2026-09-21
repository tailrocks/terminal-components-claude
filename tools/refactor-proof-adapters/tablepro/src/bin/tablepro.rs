//! Production CLI entry used by the `TablePro` oracle adapter.
//!
//! Same process as `apps/tablepro/src/main.rs`: argument parsing, help, and
//! `run_with` all belong to `tablepro_app::run`.

fn main() -> std::process::ExitCode {
    use std::io::Write as _;
    match tablepro_app::run() {
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
