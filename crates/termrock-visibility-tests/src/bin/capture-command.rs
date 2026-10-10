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
use sha2::{Digest, Sha256};
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
fn execute_request(
    request: Request,
    request_sha256: String,
) -> Result<(u8, serde_json::Value), String> {
    execute_request_with_signal(request, request_sha256, &RECEIVED_PARENT_SIGNAL)
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
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(unix)]
fn install_parent_signal_handlers() -> io::Result<()> {
    let mut action: libc::sigaction = unsafe { std::mem::zeroed() };
    action.sa_sigaction = record_parent_signal as usize;
    action.sa_flags = 0;
    // SAFETY: the signal set is writable storage initialized by libc.
    if unsafe { libc::sigemptyset(&mut action.sa_mask) } != 0 {
        return Err(io::Error::last_os_error());
    }
    for signal in [libc::SIGINT, libc::SIGTERM, libc::SIGHUP] {
        // SAFETY: `action` contains a plain atomic-only handler and a valid mask.
        if unsafe { libc::sigaction(signal, &action, std::ptr::null_mut()) } != 0 {
            return Err(io::Error::last_os_error());
        }
    }
    Ok(())
}

#[cfg(unix)]
extern "C" fn record_parent_signal(signal: libc::c_int) {
    // AtomicI32 operations are lock-free on the supported Unix targets. The
    // signal handler performs no allocation, I/O, or process management.
    let _ =
        RECEIVED_PARENT_SIGNAL.compare_exchange(0, signal, Ordering::Relaxed, Ordering::Relaxed);
}

#[cfg(all(test, unix))]
mod tests {
    use super::{
        Request, execute_request, execute_request_with_signal, parse_request, persist_receipt,
    };
    use serde_json::json;
    use std::collections::BTreeMap;
    use std::fs;
    use std::sync::atomic::AtomicI32;
    use std::time::Duration;

    #[cfg(unix)]
    #[test]
    fn request_runs_exact_argv_in_cleared_environment_and_binds_raw_receipt() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let stdout_path = directory.path().join("stdout.raw");
        let stderr_path = directory.path().join("stderr.raw");
        let result_path = directory.path().join("result.json");
        let request = json!({
            "schema": "termrock-command-capture-request/v1",
            "argv": ["/usr/bin/printf", "%s", "literal; with spaces"],
            "cwd": directory.path().display().to_string(),
            "env": {"CAPTURE_COMMAND_ONLY": "visible"},
            "timeout_ms": 5000,
            "stdout_path": stdout_path.display().to_string(),
            "stderr_path": stderr_path.display().to_string(),
            "result_path": result_path.display().to_string(),
        });
        let bytes = serde_json::to_vec(&request).expect("request JSON");
        let parsed = parse_request(&bytes).expect("valid request");
        let request_sha256 = super::hex_sha256(&bytes);
        let (status, receipt) = execute_request(parsed, request_sha256).expect("capture");

        assert_eq!(status, 0);
        assert_eq!(
            fs::read(&stdout_path).expect("stdout bytes"),
            b"literal; with spaces"
        );
        assert!(fs::read(&stderr_path).expect("stderr bytes").is_empty());
        assert_eq!(receipt["outcome"], "complete");
        assert_eq!(receipt["child"]["exit_code"], 0);
        assert_eq!(receipt["stdout"]["byte_length"], 20);
        assert_eq!(
            receipt["stdout"]["sha256"],
            super::hex_sha256(b"literal; with spaces")
        );
        assert_eq!(
            fs::read(&result_path).expect("receipt bytes").last(),
            Some(&b'\n')
        );
    }

    #[cfg(unix)]
    #[test]
    fn request_clears_inherited_environment_before_setting_requested_values() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let stdout_path = directory.path().join("stdout.raw");
        let stderr_path = directory.path().join("stderr.raw");
        let result_path = directory.path().join("result.json");
        let request = Request {
            argv: vec!["/usr/bin/env".to_owned()],
            cwd: directory.path().to_path_buf(),
            environment: BTreeMap::from([(
                "CAPTURE_COMMAND_ONLY".to_owned(),
                "visible".to_owned(),
            )]),
            timeout: Duration::from_secs(5),
            stdout_path: stdout_path.clone(),
            stderr_path: stderr_path.clone(),
            result_path: result_path.clone(),
        };
        let (status, receipt) =
            execute_request(request, "request-hash".to_owned()).expect("capture");
        let stdout = fs::read(&stdout_path).expect("stdout bytes");

        assert_eq!(status, 0);
        assert_eq!(stdout, b"CAPTURE_COMMAND_ONLY=visible\n");
        assert_eq!(receipt["env_cleared"], true);
        assert_eq!(receipt["env_keys"][0], "CAPTURE_COMMAND_ONLY");
        assert!(fs::read(&stderr_path).expect("stderr bytes").is_empty());
        assert!(result_path.is_file());
    }

    #[cfg(unix)]
    #[test]
    fn request_uses_the_explicit_working_directory() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let cwd = directory.path().canonicalize().expect("canonical cwd");
        let stdout_path = directory.path().join("stdout.raw");
        let stderr_path = directory.path().join("stderr.raw");
        let result_path = directory.path().join("result.json");
        let request = Request {
            argv: vec!["/bin/pwd".to_owned()],
            cwd: cwd.clone(),
            environment: BTreeMap::new(),
            timeout: Duration::from_secs(5),
            stdout_path: stdout_path.clone(),
            stderr_path,
            result_path,
        };
        let (status, receipt) = execute_request(request, "request-hash".to_owned())
            .expect("capture command in requested working directory");

        assert_eq!(status, 0);
        assert_eq!(receipt["outcome"], "complete");
        assert_eq!(receipt["cwd"], cwd.display().to_string());
        assert_eq!(
            fs::read(&stdout_path).expect("pwd stdout"),
            format!("{}\n", cwd.display()).as_bytes()
        );
    }

    #[cfg(unix)]
    #[test]
    fn nonzero_child_exit_is_a_complete_capture_not_wrapper_failure() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let stdout_path = directory.path().join("stdout.raw");
        let stderr_path = directory.path().join("stderr.raw");
        let result_path = directory.path().join("result.json");
        let request = Request {
            argv: vec!["/usr/bin/false".to_owned()],
            cwd: directory.path().to_path_buf(),
            environment: BTreeMap::new(),
            timeout: Duration::from_secs(5),
            stdout_path: stdout_path.clone(),
            stderr_path: stderr_path.clone(),
            result_path: result_path.clone(),
        };
        let (status, receipt) = execute_request(request, "request-hash".to_owned())
            .expect("capture nonzero child exit");

        assert_eq!(status, 0, "wrapper success means capture completed");
        assert_eq!(receipt["outcome"], "complete");
        assert_eq!(receipt["child"]["exit_code"], 1);
        assert_eq!(receipt["child"]["signal"], serde_json::Value::Null);
        assert!(stdout_path.is_file());
        assert!(stderr_path.is_file());
        assert!(result_path.is_file());
    }

    #[cfg(unix)]
    #[test]
    fn raw_stream_write_failure_is_reported_as_capture_error() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let stdout_path = directory.path().join("stdout.raw");
        let stderr_path = directory.path().join("stderr.raw");
        let result_path = directory.path().join("result.json");
        fs::write(&stdout_path, b"existing bytes").expect("seed existing stdout path");
        let request = Request {
            argv: vec!["/usr/bin/true".to_owned()],
            cwd: directory.path().to_path_buf(),
            environment: BTreeMap::new(),
            timeout: Duration::from_secs(5),
            stdout_path: stdout_path.clone(),
            stderr_path: stderr_path.clone(),
            result_path: result_path.clone(),
        };

        let (status, receipt) =
            execute_request(request, "request-hash".to_owned()).expect("capture failure");

        assert_eq!(status, 125);
        assert_eq!(receipt["outcome"], "capture_error");
        assert_eq!(receipt["stdout"]["file_written"], false);
        assert_eq!(
            fs::read(&stdout_path).expect("preserved stdout path"),
            b"existing bytes"
        );
        assert!(stderr_path.is_file());
        assert!(result_path.is_file());
    }

    #[cfg(unix)]
    #[test]
    fn request_reports_parent_cancellation_separately_from_child_status() {
        use std::sync::Arc;
        use std::sync::atomic::Ordering;

        let directory = tempfile::tempdir().expect("temporary directory");
        let stdout_path = directory.path().join("stdout.raw");
        let stderr_path = directory.path().join("stderr.raw");
        let result_path = directory.path().join("result.json");
        let signal = Arc::new(AtomicI32::new(0));
        let signal_sender = Arc::clone(&signal);
        let sender = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(50));
            signal_sender.store(libc::SIGTERM, Ordering::Release);
        });
        let request = Request {
            argv: vec!["/bin/sleep".to_owned(), "30".to_owned()],
            cwd: directory.path().to_path_buf(),
            environment: BTreeMap::new(),
            timeout: Duration::from_secs(5),
            stdout_path,
            stderr_path,
            result_path: result_path.clone(),
        };
        let (status, receipt) =
            execute_request_with_signal(request, "request-hash".to_owned(), &signal)
                .expect("cancelled capture");
        sender.join().expect("signal thread");

        assert_eq!(status, 143);
        assert_eq!(receipt["outcome"], "cancelled");
        assert_eq!(receipt["parent_signal"], libc::SIGTERM);
        assert_eq!(receipt["child"]["signal"], libc::SIGKILL);
        assert_eq!(receipt["child"]["timed_out"], false);
        assert!(result_path.is_file());
    }

    #[cfg(unix)]
    #[test]
    fn rejects_nonabsolute_executable_and_unbounded_timeout() {
        let request = json!({
            "schema": "termrock-command-capture-request/v1",
            "argv": ["echo", "unsafe PATH lookup"],
            "cwd": "/tmp",
            "env": {},
            "timeout_ms": 5000,
            "stdout_path": "/tmp/capture-command-out",
            "stderr_path": "/tmp/capture-command-err",
            "result_path": "/tmp/capture-command-result",
        });
        assert!(parse_request(&serde_json::to_vec(&request).expect("request JSON")).is_err());

        let mut request = request;
        request["argv"] = json!(["/usr/bin/true"]);
        request["timeout_ms"] = json!(u64::MAX);
        assert!(parse_request(&serde_json::to_vec(&request).expect("request JSON")).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn result_receipt_write_failure_is_a_capture_error() {
        let directory = tempfile::tempdir().expect("temporary directory");
        let missing_result = directory.path().join("missing/result.json");
        let receipt = serde_json::json!({"outcome": "complete"});

        let (status, receipt) = persist_receipt(&missing_result, receipt, 0);

        assert_eq!(status, 125);
        assert_eq!(receipt["outcome"], "capture_error");
        assert_eq!(receipt["result_file_written"], false);
        assert!(
            receipt["capture_error"]
                .as_str()
                .expect("capture error text")
                .contains("cannot write result receipt")
        );
    }
}
