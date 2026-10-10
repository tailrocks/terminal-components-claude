use std::env;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

use termrock_visibility_tests::nextest_result::read_files;

const USAGE: &str = "usage: nextest-result-reader --request REQUEST.json --selection SELECTION.json --trusted-selection-sha256 SHA256";

fn main() -> ExitCode {
    match run(env::args_os().skip(1)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("nextest-result-reader: {message}");
            ExitCode::from(2)
        }
    }
}

fn run(arguments: impl IntoIterator<Item = OsString>) -> Result<(), String> {
    let arguments = arguments.into_iter().collect::<Vec<_>>();
    if arguments.as_slice() == [OsString::from("--help")] {
        println!("{USAGE}");
        return Ok(());
    }
    if arguments.len() != 6
        || arguments[0] != "--request"
        || arguments[2] != "--selection"
        || arguments[4] != "--trusted-selection-sha256"
    {
        return Err(USAGE.to_owned());
    }
    let request_path = PathBuf::from(&arguments[1]);
    let selection_path = PathBuf::from(&arguments[3]);
    let trusted_sha256 = arguments[5]
        .to_str()
        .ok_or_else(|| "trusted SHA-256 argument is not valid UTF-8".to_owned())?;
    let report = read_files(&request_path, &selection_path, trusted_sha256)
        .map_err(|error| error.to_string())?;
    serde_json::to_writer(std::io::stdout().lock(), &report)
        .map_err(|error| format!("cannot write report JSON: {error}"))?;
    println!();
    Ok(())
}
