#[cfg(unix)]
use std::collections::BTreeMap;
#[cfg(unix)]
use std::env;
#[cfg(unix)]
use std::fs::{self, OpenOptions};
#[cfg(unix)]
use std::io::{self, Write};
#[cfg(unix)]
use std::path::{Path, PathBuf};
#[cfg(unix)]
use std::process::{Command, ExitCode};
#[cfg(unix)]
use std::sync::atomic::{AtomicI32, Ordering};
#[cfg(unix)]
use std::time::Duration;

#[cfg(unix)]
use termrock_visibility_tests::run_command_with_cancellation;

#[cfg(unix)]
const USAGE: &str = "usage: capture-command --request REQUEST.json";
#[cfg(unix)]
const REQUEST_SCHEMA: &str = "termrock-command-capture-request/v1";
#[cfg(unix)]
const RESULT_SCHEMA: &str = "termrock-command-capture-result/v1";
#[cfg(unix)]
const MAX_TIMEOUT_MS: u64 = 24 * 60 * 60 * 1000;

#[cfg(unix)]
static RECEIVED_PARENT_SIGNAL: AtomicI32 = AtomicI32::new(0);

#[cfg(unix)]
#[derive(Debug)]
enum RunError {
    Input(String),
    Capture(String),
}

#[cfg(unix)]
#[derive(Debug)]
struct Request {
    argv: Vec<String>,
    cwd: PathBuf,
    environment: BTreeMap<String, String>,
    timeout: Duration,
    stdout_path: PathBuf,
    stderr_path: PathBuf,
    result_path: PathBuf,
}

#[cfg(unix)]
fn main() -> ExitCode {
    let arguments = env::args_os().skip(1).collect::<Vec<_>>();
    if arguments.as_slice() == [std::ffi::OsString::from("--help")] {
        println!("{USAGE}");
        return ExitCode::SUCCESS;
    }
    match run(arguments) {
        Ok((status, receipt)) => {
            let mut stdout = io::stdout().lock();
            if let Err(error) = write_json_line(&mut stdout, &receipt) {
                eprintln!("capture-command: cannot write receipt to stdout: {error}");
                return ExitCode::from(125);
            }
            ExitCode::from(status)
        }
        Err(RunError::Input(message)) => {
            eprintln!("capture-command: {message}");
            ExitCode::from(2)
        }
        Err(RunError::Capture(message)) => {
            eprintln!("capture-command: capture failed: {message}");
            ExitCode::from(125)
        }
    }
}

#[cfg(not(unix))]
fn main() -> std::process::ExitCode {
    eprintln!("capture-command: bounded process-group capture requires Unix");
    std::process::ExitCode::from(2)
}

#[cfg(unix)]
fn run(
    arguments: impl IntoIterator<Item = std::ffi::OsString>,
) -> Result<(u8, serde_json::Value), RunError> {
    let arguments = arguments.into_iter().collect::<Vec<_>>();
    if arguments.len() != 2 || arguments[0] != "--request" {
        return Err(RunError::Input(USAGE.to_owned()));
    }

    let request_path = PathBuf::from(&arguments[1]);
    let (request, request_sha256, canonical_request_path) =
        load_request(&request_path).map_err(RunError::Input)?;
    validate_output_paths(&request, &canonical_request_path).map_err(RunError::Input)?;
    install_parent_signal_handlers()
        .map_err(|error| RunError::Capture(format!("cannot install signal handlers: {error}")))?;
    execute_request_with_signal(request, request_sha256, &RECEIVED_PARENT_SIGNAL)
        .map_err(RunError::Capture)
}

#[cfg(unix)]
fn load_request(path: &Path) -> Result<(Request, String, PathBuf), String> {
    let canonical_path = fs::canonicalize(path)
        .map_err(|error| format!("cannot resolve request path {}: {error}", path.display()))?;
    let bytes = fs::read(&canonical_path)
        .map_err(|error| format!("cannot read request {}: {error}", canonical_path.display()))?;
    let request = parse_request(&bytes)?;
    let request_sha256 = hex_sha256(&bytes);
    Ok((request, request_sha256, canonical_path))
}

#[cfg(unix)]
fn parse_request(bytes: &[u8]) -> Result<Request, String> {
    use serde_json::Value;

    let value: Value = serde_json::from_slice(bytes)
        .map_err(|error| format!("request is not valid JSON: {error}"))?;
    let object = value
        .as_object()
        .ok_or_else(|| "request root must be an object".to_owned())?;
    const FIELDS: [&str; 8] = [
        "schema",
        "argv",
        "cwd",
        "env",
        "timeout_ms",
        "stdout_path",
        "stderr_path",
        "result_path",
    ];
    if object.len() != FIELDS.len() || object.keys().any(|key| !FIELDS.contains(&key.as_str())) {
        return Err("request fields do not match the v1 schema".to_owned());
    }
    if object.get("schema").and_then(Value::as_str) != Some(REQUEST_SCHEMA) {
        return Err(format!("request schema must be {REQUEST_SCHEMA}"));
    }

    let argv = object
        .get("argv")
        .and_then(Value::as_array)
        .ok_or_else(|| "argv must be an array of strings".to_owned())?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| "argv must contain only strings".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    if argv.is_empty() || argv.iter().any(|argument| argument.contains('\0')) {
        return Err("argv must be nonempty and contain no NUL bytes".to_owned());
    }
    let executable = Path::new(&argv[0]);
    if !executable.is_absolute() {
        return Err("argv[0] must be an absolute executable path".to_owned());
    }

    let cwd = absolute_path_member(object, "cwd")?;
    let stdout_path = absolute_path_member(object, "stdout_path")?;
    let stderr_path = absolute_path_member(object, "stderr_path")?;
    let result_path = absolute_path_member(object, "result_path")?;
    let timeout_ms = object
        .get("timeout_ms")
        .and_then(Value::as_u64)
        .filter(|milliseconds| (1..=MAX_TIMEOUT_MS).contains(milliseconds))
        .ok_or_else(|| format!("timeout_ms must be between 1 and {MAX_TIMEOUT_MS}"))?;

    let environment_object = object
        .get("env")
        .and_then(Value::as_object)
        .ok_or_else(|| "env must be an object of string values".to_owned())?;
    let mut environment = BTreeMap::new();
    for (name, value) in environment_object {
        let value = value
            .as_str()
            .ok_or_else(|| format!("environment value for {name:?} must be a string"))?;
        if name.is_empty() || name.contains('=') || name.contains('\0') || value.contains('\0') {
            return Err(format!("invalid environment entry {name:?}"));
        }
        environment.insert(name.clone(), value.to_owned());
    }

    Ok(Request {
        argv,
        cwd,
        environment,
        timeout: Duration::from_millis(timeout_ms),
        stdout_path,
        stderr_path,
        result_path,
    })
}

#[cfg(unix)]
fn absolute_path_member(
    object: &serde_json::Map<String, serde_json::Value>,
    name: &str,
) -> Result<PathBuf, String> {
    let value = object
        .get(name)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| format!("{name} must be a path string"))?;
    if value.contains('\0') {
        return Err(format!("{name} must not contain NUL bytes"));
    }
    let path = PathBuf::from(value);
    if !path.is_absolute() {
        return Err(format!("{name} must be an absolute path"));
    }
    Ok(path)
}

#[cfg(unix)]
fn validate_output_paths(request: &Request, request_path: &Path) -> Result<(), String> {
    let paths = [
        (&request.stdout_path, "stdout_path"),
        (&request.stderr_path, "stderr_path"),
        (&request.result_path, "result_path"),
    ];
    let mut canonical_paths = Vec::with_capacity(paths.len());
    for (path, name) in paths {
        let parent = path
            .parent()
            .ok_or_else(|| format!("{name} has no parent directory"))?;
        let leaf = path
            .file_name()
            .ok_or_else(|| format!("{name} must name a file"))?;
        let parent = fs::canonicalize(parent)
            .map_err(|error| format!("cannot resolve {name} parent: {error}"))?;
        let canonical = parent.join(leaf);
        if canonical == request_path {
            return Err(format!("{name} must not overwrite the request file"));
        }
        match fs::symlink_metadata(&canonical) {
            Ok(_) => return Err(format!("{name} already exists: {}", canonical.display())),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(format!("cannot inspect {name}: {error}")),
        }
        canonical_paths.push(canonical);
    }
    if canonical_paths[0] == canonical_paths[1]
        || canonical_paths[0] == canonical_paths[2]
        || canonical_paths[1] == canonical_paths[2]
    {
        return Err("stdout, stderr, and result paths must be distinct".to_owned());
    }
    Ok(())
}

#[cfg(unix)]
fn execute_request_with_signal(
    request: Request,
    request_sha256: String,
    parent_signal: &AtomicI32,
) -> Result<(u8, serde_json::Value), String> {
    let mut command = Command::new(&request.argv[0]);
    command
        .args(&request.argv[1..])
        .current_dir(&request.cwd)
        .env_clear();
    for (name, value) in &request.environment {
        command.env(name, value);
    }

    let captured =
        match run_command_with_cancellation(command, request.timeout, Some(parent_signal)) {
            Ok(captured) => captured,
            Err(error) => {
                let receipt = serde_json::json!({
                    "schema": RESULT_SCHEMA,
                    "request_sha256": request_sha256,
                    "argv": request.argv,
                    "cwd": request.cwd.display().to_string(),
                    "env_cleared": true,
                    "env_keys": request.environment.keys().collect::<Vec<_>>(),
                    "timeout_ms": request.timeout.as_millis() as u64,
                    "outcome": "capture_error",
                    "capture_error": error.to_string(),
                    "child": serde_json::Value::Null,
                    "stdout": serde_json::Value::Null,
                    "stderr": serde_json::Value::Null,
                });
                return Ok(persist_receipt(&request.result_path, receipt, 125));
            }
        };

    let cancellation_signal = captured.cancellation_signal.or_else(|| {
        let signal = parent_signal.load(Ordering::Acquire);
        (signal > 0).then_some(signal)
    });
    let output = captured.output;
    let stdout_write = write_new_file(&request.stdout_path, &output.stdout);
    let stderr_write = write_new_file(&request.stderr_path, &output.stderr);
    let write_errors = [
        stdout_write
            .as_ref()
            .err()
            .map(|error| format!("stdout: {error}")),
        stderr_write
            .as_ref()
            .err()
            .map(|error| format!("stderr: {error}")),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    if !write_errors.is_empty() {
        let receipt = serde_json::json!({
            "schema": RESULT_SCHEMA,
            "request_sha256": request_sha256,
            "argv": request.argv,
            "cwd": request.cwd.display().to_string(),
            "env_cleared": true,
            "env_keys": request.environment.keys().collect::<Vec<_>>(),
            "timeout_ms": request.timeout.as_millis() as u64,
            "outcome": "capture_error",
            "capture_error": write_errors.join("; "),
            "parent_signal": cancellation_signal,
            "child": {
                "exit_code": output.exit_code,
                "signal": output.signal,
                "timed_out": output.timed_out,
            },
            "stdout": stream_write_receipt(
                &request.stdout_path,
                &output.stdout,
                output.stdout_truncated,
                stdout_write.is_ok(),
            ),
            "stderr": stream_write_receipt(
                &request.stderr_path,
                &output.stderr,
                output.stderr_truncated,
                stderr_write.is_ok(),
            ),
        });
        return Ok(persist_receipt(&request.result_path, receipt, 125));
    }

    let outcome = if cancellation_signal.is_some() {
        "cancelled"
    } else if output.timed_out {
        "timed_out"
    } else {
        "complete"
    };
    let receipt = serde_json::json!({
        "schema": RESULT_SCHEMA,
        "request_sha256": request_sha256,
        "argv": request.argv,
        "cwd": request.cwd.display().to_string(),
        "env_cleared": true,
        "env_keys": request.environment.keys().collect::<Vec<_>>(),
        "timeout_ms": request.timeout.as_millis() as u64,
        "outcome": outcome,
        "parent_signal": cancellation_signal,
        "child": {
            "exit_code": output.exit_code,
            "signal": output.signal,
            "timed_out": output.timed_out,
        },
        "stdout": stream_write_receipt(
            &request.stdout_path,
            &output.stdout,
            output.stdout_truncated,
            true,
        ),
        "stderr": stream_write_receipt(
            &request.stderr_path,
            &output.stderr,
            output.stderr_truncated,
            true,
        ),
    });
    let cli_status = match cancellation_signal {
        Some(signal) => 128_u8.saturating_add(signal.min(127) as u8),
        None if output.timed_out => 124,
        None => 0,
    };
    Ok(persist_receipt(&request.result_path, receipt, cli_status))
}

#[cfg(unix)]
fn persist_receipt(
    path: &Path,
    mut receipt: serde_json::Value,
    completed_status: u8,
) -> (u8, serde_json::Value) {
    match write_receipt(path, &receipt) {
        Ok(()) => (completed_status, receipt),
        Err(error) => {
            let prior_error = receipt
                .get("capture_error")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned);
            receipt["outcome"] = serde_json::Value::String("capture_error".to_owned());
            receipt["result_file_written"] = serde_json::Value::Bool(false);
            receipt["capture_error"] = serde_json::Value::String(match prior_error {
                Some(prior_error) => format!(
                    "{prior_error}; cannot write result receipt {}: {error}",
                    path.display()
                ),
                None => format!("cannot write result receipt {}: {error}", path.display()),
            });
            (125, receipt)
        }
    }
}

#[cfg(unix)]
fn stream_receipt(path: &Path, bytes: &[u8], truncated: bool) -> serde_json::Value {
    serde_json::json!({
        "path": path.display().to_string(),
        "byte_length": bytes.len(),
        "sha256": hex_sha256(bytes),
        "truncated": truncated,
    })
}

#[cfg(unix)]
fn stream_write_receipt(
    path: &Path,
    bytes: &[u8],
    truncated: bool,
    file_written: bool,
) -> serde_json::Value {
    let mut receipt = stream_receipt(path, bytes, truncated);
    receipt["file_written"] = serde_json::Value::Bool(file_written);
    receipt
}

#[cfg(unix)]
fn write_receipt(path: &Path, receipt: &serde_json::Value) -> Result<(), String> {
    let mut bytes = serde_json::to_vec(receipt)
        .map_err(|error| format!("cannot encode result receipt: {error}"))?;
    bytes.push(b'\n');
    write_new_file(path, &bytes)
        .map_err(|error| format!("cannot write result receipt {}: {error}", path.display()))
}

#[cfg(unix)]
fn write_json_line(writer: &mut impl Write, value: &serde_json::Value) -> io::Result<()> {
    serde_json::to_writer(&mut *writer, value)?;
    writer.write_all(b"\n")?;
    writer.flush()
}

#[cfg(unix)]
fn write_new_file(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

#[cfg(unix)]
fn hex_sha256(bytes: &[u8]) -> String {
    termrock_visibility_tests::sha256_hex(bytes)
}

#[cfg(unix)]
fn install_parent_signal_handlers() -> io::Result<()> {
    let mut action: libc::sigaction = unsafe { std::mem::zeroed() };
    action.sa_sigaction = record_parent_signal as *const () as usize;
    action.sa_flags = 0;

    if unsafe { libc::sigemptyset(&mut action.sa_mask) } != 0 {
        return Err(io::Error::last_os_error());
    }
    for signal in [libc::SIGINT, libc::SIGTERM, libc::SIGHUP] {
        if unsafe { libc::sigaction(signal, &action, std::ptr::null_mut()) } != 0 {
            return Err(io::Error::last_os_error());
        }
    }
    Ok(())
}

#[cfg(unix)]
extern "C" fn record_parent_signal(signal: libc::c_int) {
    let _ =
        RECEIVED_PARENT_SIGNAL.compare_exchange(0, signal, Ordering::Relaxed, Ordering::Relaxed);
}
