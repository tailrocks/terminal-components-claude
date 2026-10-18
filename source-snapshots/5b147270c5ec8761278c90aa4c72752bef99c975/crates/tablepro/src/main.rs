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

#[cfg(test)]
mod tests {
    #[test]
    fn test_tablepro_binary_entry() {
        // Sanity check verifying tablepro binary compiles and links.
    }
}
