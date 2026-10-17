//! Synthetic rustup, Cargo, and Nextest command protocols for CLI tests.
//!
//! The test modules install this compiled binary under command names such as
//! `rustup`, `mise`, and `cargo-nextest`. Its output is synthetic and cannot
//! qualify or stand in for a Termrock product executable.

use std::env;
use std::ffi::{OsStr, OsString};
use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

const TRACE_ENV: &str = "VISIBILITY_FIXTURE_TRACE";
const NEXTTEST_LIST_ENV: &str = "VISIBILITY_FIXTURE_NEXTTEST_LIST";
const NEXTTEST_LIST_EXIT_ENV: &str = "VISIBILITY_FIXTURE_NEXTTEST_LIST_EXIT";
const NEXTTEST_RUN_ENV: &str = "VISIBILITY_FIXTURE_NEXTTEST_RUN";
const NEXTTEST_EXIT_ENV: &str = "VISIBILITY_FIXTURE_NEXTTEST_EXIT";
const NEXTTEST_SIGNAL_ENV: &str = "VISIBILITY_FIXTURE_NEXTTEST_SIGNAL";
const NEXTTEST_VERSION_ENV: &str = "VISIBILITY_FIXTURE_NEXTEST_VERSION";
const NEXTTEST_VERSION_EXIT_ENV: &str = "VISIBILITY_FIXTURE_NEXTEST_VERSION_EXIT";
const CARGO_METADATA_ENV: &str = "VISIBILITY_FIXTURE_CARGO_METADATA";
const CARGO_METADATA_RAW_ENV: &str = "VISIBILITY_FIXTURE_CARGO_METADATA_RAW";
const CARGO_METADATA_EXIT_ENV: &str = "VISIBILITY_FIXTURE_CARGO_METADATA_EXIT";
const CARGO_METADATA_STDERR_ENV: &str = "VISIBILITY_FIXTURE_CARGO_METADATA_STDERR";
const CARGO_BUILD_EXIT_ENV: &str = "VISIBILITY_FIXTURE_CARGO_BUILD_EXIT";
const CARGO_BUILD_STDOUT_ENV: &str = "VISIBILITY_FIXTURE_CARGO_BUILD_STDOUT";
const CARGO_BUILD_STDERR_ENV: &str = "VISIBILITY_FIXTURE_CARGO_BUILD_STDERR";
const ARTIFACT_MODE_ENV: &str = "VISIBILITY_FIXTURE_ARTIFACT_MODE";
const CARGO_VERSION_ENV: &str = "VISIBILITY_FIXTURE_CARGO_VERSION";
const RUSTC_VERSION_ENV: &str = "VISIBILITY_FIXTURE_RUSTC_VERSION";
const RUSTC_VERBOSE_ENV: &str = "VISIBILITY_FIXTURE_RUSTC_VERBOSE";
const RUSTUP_HOME_ENV: &str = "VISIBILITY_FIXTURE_RUSTUP_HOME";

fn main() {
    let mut raw_args = env::args_os();
    let invoked_as = raw_args.next().unwrap_or_else(|| OsString::from("fixture"));
    let args: Vec<OsString> = raw_args.collect();
    let program = command_name(&invoked_as);
    record_invocation(&program, &args);

    let exit_code = match program.as_str() {
        "rustup" => rustup(&args),
        "mise" => mise(&args),
        "cargo" => cargo(&strings(&args)),
        "rustc" => rustc(&strings(&args)),
        "cargo-nextest" => cargo_nextest(&args),
        "fixture" => fixture_help(),
        other => fail(format!("unknown fixture command name: {other}")),
    };
    std::process::exit(exit_code);
}

fn command_name(path: &OsStr) -> String {
    let name = Path::new(path)
        .file_name()
        .unwrap_or(path)
        .to_string_lossy();
    name.strip_suffix(".exe").unwrap_or(&name).to_owned()
}

fn record_invocation(program: &str, args: &[OsString]) {
    let Some(path) = env::var_os(TRACE_ENV).map(PathBuf::from) else {
        return;
    };
    let argv: Vec<String> = args
        .iter()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect();
    let cwd = env::current_dir()
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_else(|_| "<unknown>".to_owned());
    if let Err(error) = append_json_line(
        &path,
        &json!({
            "program": program,
            "argv": argv,
            "cwd": cwd,
        }),
    ) {
        eprintln!("fixture trace error: {error}");
    }
}

fn append_json_line(path: &Path, value: &Value) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    serde_json::to_writer(&mut file, value)?;
    file.write_all(b"\n")
}

fn rustup(args: &[OsString]) -> i32 {
    let args = strings(args);
    match args.as_slice() {
        [verb, flag, toolchain, program]
            if verb == "which"
                && flag == "--toolchain"
                && toolchain == "1.98.1"
                && (program == "cargo" || program == "rustc") =>
        {
            print_alias_path(program)
        }
        [verb, home] if verb == "show" && home == "home" => {
            let path = env::var(RUSTUP_HOME_ENV).unwrap_or_else(|_| {
                env::temp_dir()
                    .join("visibility-rustup-home")
                    .to_string_lossy()
                    .into_owned()
            });
            println!("{path}");
            0
        }
        [verb, toolchain, program, rest @ ..] if verb == "run" && toolchain == "1.98.1" => {
            match program.as_str() {
                "cargo" => cargo(rest),
                "rustc" => rustc(rest),
                other => fail(format!("unsupported rustup run program: {other}")),
            }
        }
        _ => fail(format!("unsupported rustup argv: {args:?}")),
    }
}

fn mise(args: &[OsString]) -> i32 {
    let args = strings(args);
    match args.as_slice() {
        [verb, executable] if verb == "which" && executable == "cargo-nextest" => {
            print_alias_path("cargo-nextest")
        }
        _ => fail(format!("unsupported mise argv: {args:?}")),
    }
}

fn cargo_nextest(args: &[OsString]) -> i32 {
    let args = strings(args);
    if args.as_slice() == ["--version"] {
        nextest_version()
    } else {
        fail(format!("unsupported cargo-nextest argv: {args:?}"))
    }
}

fn cargo(args: &[String]) -> i32 {
    match args {
        [version] if version == "--version" => {
            println!(
                "{}",
                env::var(CARGO_VERSION_ENV)
                    .unwrap_or_else(|_| "cargo 1.98.1 (synthetic visibility fixture)".to_owned())
            );
            parse_exit("VISIBILITY_FIXTURE_CARGO_VERSION_EXIT", 0)
        }
        [nextest, version] if nextest == "nextest" && version == "--version" => nextest_version(),
        [nextest, action, rest @ ..] if nextest == "nextest" && action == "list" => {
            let _ = rest;
            emit_file(NEXTTEST_LIST_ENV, NEXTTEST_LIST_EXIT_ENV, 0)
        }
        [nextest, action, rest @ ..] if nextest == "nextest" && action == "run" => {
            let _ = rest;
            let signal = match requested_nextest_signal() {
                Ok(signal) => signal,
                Err(message) => return fail(message),
            };
            let result = emit_file(NEXTTEST_RUN_ENV, NEXTTEST_EXIT_ENV, 0);
            if let Some(signal) = signal {
                if let Err(error) = terminate_by_signal(signal) {
                    return fail(format!(
                        "cannot terminate fixture with signal {signal}: {error}"
                    ));
                }
            }
            result
        }
        [metadata, rest @ ..] if metadata == "metadata" => cargo_metadata(rest),
        [build, rest @ ..] if build == "build" => cargo_build(rest),
        _ => fail(format!("unsupported cargo argv: {args:?}")),
    }
}

fn rustc(args: &[String]) -> i32 {
    match args {
        [version] if version == "--version" => {
            println!(
                "{}",
                env::var(RUSTC_VERSION_ENV)
                    .unwrap_or_else(|_| "rustc 1.98.1 (synthetic visibility fixture)".to_owned())
            );
            parse_exit("VISIBILITY_FIXTURE_RUSTC_VERSION_EXIT", 0)
        }
        [verbose] if verbose == "-Vv" => {
            let output = env::var(RUSTC_VERBOSE_ENV).unwrap_or_else(|_| {
                format!(
                    "rustc 1.98.1 (synthetic visibility fixture)\nbinary: rustc\ncommit-hash: fixture\ncommit-date: 2026-10-08\nhost: {}\nrelease: 1.98.1\nLLVM version: fixture\n",
                    host_triple()
                )
            });
            print!("{output}");
            parse_exit("VISIBILITY_FIXTURE_RUSTC_VERBOSE_EXIT", 0)
        }
        _ => fail(format!("unsupported rustc argv: {args:?}")),
    }
}

fn cargo_metadata(args: &[String]) -> i32 {
    emit_env_file(CARGO_METADATA_STDERR_ENV, io::stderr());
    if let Some(raw) = env::var_os(CARGO_METADATA_RAW_ENV) {
        if let Err(error) = copy_file_to_stdout(Path::new(&raw)) {
            return fail(format!("cannot emit raw Cargo metadata: {error}"));
        }
        return parse_exit(CARGO_METADATA_EXIT_ENV, 0);
    }

    let Some(input) = env::var_os(CARGO_METADATA_ENV) else {
        return fail(format!(
            "{CARGO_METADATA_ENV} is required for fake Cargo metadata"
        ));
    };
    let manifest = match value_after(args, "--manifest-path") {
        Some(path) => PathBuf::from(path),
        None => return fail("Cargo metadata argv has no --manifest-path".to_owned()),
    };
    let Some(workspace_root) = manifest.parent() else {
        return fail("Cargo manifest has no parent directory".to_owned());
    };
    let bytes = match fs::read(input) {
        Ok(bytes) => bytes,
        Err(error) => return fail(format!("cannot read fake Cargo metadata: {error}")),
    };
    let mut metadata: Value = match serde_json::from_slice(&bytes) {
        Ok(value) => value,
        Err(error) => return fail(format!("fake Cargo metadata JSON is invalid: {error}")),
    };
    let replacements = [
        (
            "__WORKSPACE_ROOT__",
            workspace_root.to_string_lossy().into_owned(),
        ),
        ("__MANIFEST_PATH__", manifest.to_string_lossy().into_owned()),
    ];
    replace_tokens(&mut metadata, &replacements);
    if let Err(error) = serde_json::to_writer(io::stdout().lock(), &metadata) {
        return fail(format!("cannot write fake Cargo metadata: {error}"));
    }
    println!();
    parse_exit(CARGO_METADATA_EXIT_ENV, 0)
}

fn cargo_build(args: &[String]) -> i32 {
    emit_env_file(CARGO_BUILD_STDERR_ENV, io::stderr());
    let exit = parse_exit(CARGO_BUILD_EXIT_ENV, 0);
    if exit != 0 {
        return exit;
    }
    if let Some(raw) = env::var_os(CARGO_BUILD_STDOUT_ENV) {
        if let Err(error) = copy_file_to_stdout(Path::new(&raw)) {
            return fail(format!("cannot emit raw Cargo build stdout: {error}"));
        }
        return 0;
    }
    match artifact_output(args) {
        Ok(bytes) => {
            if let Err(error) = io::stdout().write_all(&bytes) {
                return fail(format!("cannot write fake Cargo build output: {error}"));
            }
            0
        }
        Err(error) => fail(error),
    }
}

fn artifact_output(args: &[String]) -> Result<Vec<u8>, String> {
    let mode = env::var(ARTIFACT_MODE_ENV).unwrap_or_else(|_| "valid".to_owned());
    if mode == "malformed" {
        return Ok(b"not-json\n".to_vec());
    }
    if mode == "missing" {
        return Ok(Vec::new());
    }

    let metadata_path = env::var_os(CARGO_METADATA_ENV)
        .ok_or_else(|| format!("{CARGO_METADATA_ENV} is required for fake Cargo build"))?;
    let metadata_bytes = fs::read(metadata_path).map_err(|error| error.to_string())?;
    let mut metadata: Value = serde_json::from_slice(&metadata_bytes)
        .map_err(|error| format!("fake Cargo metadata JSON is invalid: {error}"))?;
    let manifest = value_after(args, "--manifest-path")
        .ok_or_else(|| "Cargo build argv has no --manifest-path".to_owned())?;
    let workspace_root = Path::new(manifest)
        .parent()
        .ok_or_else(|| "Cargo manifest has no parent directory".to_owned())?;
    replace_tokens(
        &mut metadata,
        &[
            (
                "__WORKSPACE_ROOT__",
                workspace_root.to_string_lossy().into_owned(),
            ),
            ("__MANIFEST_PATH__", manifest.to_owned()),
        ],
    );

    let binary =
        value_after(args, "--bin").ok_or_else(|| "Cargo build argv has no --bin".to_owned())?;
    let target_triple = value_after(args, "--target")
        .ok_or_else(|| "Cargo build argv has no --target".to_owned())?;
    let target_dir = PathBuf::from(
        value_after(args, "--target-dir")
            .ok_or_else(|| "Cargo build argv has no --target-dir".to_owned())?,
    );
    let (package_id, target_source) = package_target(&metadata, binary)?;
    let profile = PathBuf::from(&target_dir)
        .join(target_triple)
        .join("release");
    fs::create_dir_all(&profile).map_err(|error| error.to_string())?;
    let executable = match mode.as_str() {
        "outside-target" => {
            let parent = target_dir
                .parent()
                .ok_or_else(|| "isolated target directory has no parent".to_owned())?;
            parent.join(format!("fixture-outside-{}", binary))
        }
        _ => profile.join(format!("{binary}{}", std::env::consts::EXE_SUFFIX)),
    };
    fs::write(&executable, b"synthetic fixture executable\n").map_err(|error| error.to_string())?;

    #[cfg(unix)]
    if mode != "non-executable" {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&executable)
            .map_err(|error| error.to_string())?
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&executable, permissions).map_err(|error| error.to_string())?;
    }

    let mut event = json!({
        "reason": "compiler-artifact",
        "package_id": package_id,
        "target": {
            "name": binary,
            "kind": ["bin"],
            "crate_types": ["bin"],
            "src_path": target_source,
            "edition": "2024",
            "doc": false,
            "doctest": false,
            "test": false
        },
        "profile": {"test": false},
        "filenames": [executable],
        "executable": executable,
        "fresh": false
    });
    match mode.as_str() {
        "wrong-package" => event["package_id"] = json!("fixture-wrong-package"),
        "wrong-target" => event["target"]["name"] = json!("fixture-wrong-target"),
        "fresh" => event["fresh"] = json!(true),
        "test-profile" => event["profile"]["test"] = json!(true),
        "valid" | "duplicate" | "outside-target" | "non-executable" => {}
        other => return Err(format!("unsupported artifact fixture mode: {other}")),
    }
    let mut output = serde_json::to_vec(&event).map_err(|error| error.to_string())?;
    output.push(b'\n');
    if mode == "duplicate" {
        output.extend(serde_json::to_vec(&event).map_err(|error| error.to_string())?);
        output.push(b'\n');
    }
    Ok(output)
}

fn package_target(metadata: &Value, binary: &str) -> Result<(String, String), String> {
    let packages = metadata
        .get("packages")
        .and_then(Value::as_array)
        .ok_or_else(|| "fake Cargo metadata has no packages array".to_owned())?;
    let mut matches = Vec::new();
    for package in packages {
        let Some(package_id) = package.get("id").and_then(Value::as_str) else {
            continue;
        };
        let Some(targets) = package.get("targets").and_then(Value::as_array) else {
            continue;
        };
        for target in targets {
            if target.get("name").and_then(Value::as_str) != Some(binary) {
                continue;
            }
            let is_bin = target
                .get("kind")
                .and_then(Value::as_array)
                .is_some_and(|kinds| kinds.iter().any(|kind| kind.as_str() == Some("bin")));
            if !is_bin {
                continue;
            }
            let source = target
                .get("src_path")
                .and_then(Value::as_str)
                .ok_or_else(|| "fake Cargo target has no source path".to_owned())?;
            matches.push((package_id.to_owned(), source.to_owned()));
        }
    }
    match matches.as_slice() {
        [target] => Ok(target.clone()),
        [] => Err(format!(
            "fake Cargo metadata has no binary target named {binary}"
        )),
        _ => Err(format!(
            "fake Cargo metadata has multiple binary targets named {binary}"
        )),
    }
}

fn replace_tokens(value: &mut Value, replacements: &[(&str, String)]) {
    match value {
        Value::String(text) => {
            for (token, replacement) in replacements {
                *text = text.replace(token, replacement);
            }
        }
        Value::Array(values) => {
            for value in values {
                replace_tokens(value, replacements);
            }
        }
        Value::Object(values) => {
            for value in values.values_mut() {
                replace_tokens(value, replacements);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
}

fn print_alias_path(alias: &str) -> i32 {
    let argv0 = env::args_os()
        .next()
        .unwrap_or_else(|| OsString::from("fixture"));
    let direct = Path::new(&argv0);
    let parent = if direct.is_absolute() || direct.components().count() > 1 {
        direct.parent().map(Path::to_path_buf)
    } else {
        env::var_os("PATH")
            .and_then(|path| env::split_paths(&path).find(|entry| entry.join(direct).is_file()))
    };
    let Some(parent) = parent else {
        return fail("fixture command directory is not on PATH".to_owned());
    };
    println!(
        "{}",
        parent
            .join(format!("{alias}{}", std::env::consts::EXE_SUFFIX))
            .display()
    );
    0
}

fn emit_file(path_env: &str, exit_env: &str, default_exit: i32) -> i32 {
    let Some(path) = env::var_os(path_env).map(PathBuf::from) else {
        return fail(format!("{path_env} is required for this fixture command"));
    };
    if let Err(error) = copy_file_to_stdout(&path) {
        return fail(format!(
            "cannot read fixture output {}: {error}",
            path.display()
        ));
    }
    parse_exit(exit_env, default_exit)
}

fn emit_env_file<W: Write>(name: &str, mut output: W) {
    if let Some(path) = env::var_os(name) {
        if let Err(error) = copy_file(&PathBuf::from(path), &mut output) {
            eprintln!("cannot emit fixture stderr: {error}");
        }
    }
}

fn copy_file_to_stdout(path: &Path) -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    copy_file(path, &mut stdout)
}

fn copy_file(path: &Path, mut output: impl Write) -> io::Result<()> {
    let mut file = fs::File::open(path)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    output.write_all(&bytes)
}

fn parse_exit(name: &str, default: i32) -> i32 {
    env::var(name)
        .ok()
        .and_then(|value| value.parse::<i32>().ok())
        .filter(|code| (0..=255).contains(code))
        .unwrap_or(default)
}

fn nextest_version() -> i32 {
    println!(
        "{}",
        env::var(NEXTTEST_VERSION_ENV)
            .unwrap_or_else(|_| "cargo-nextest 0.9.146 (synthetic visibility fixture)".to_owned())
    );
    parse_exit(NEXTTEST_VERSION_EXIT_ENV, 0)
}

fn value_after<'a>(args: &'a [String], option: &str) -> Option<&'a str> {
    args.iter()
        .position(|arg| arg == option)
        .and_then(|index| args.get(index + 1))
        .map(String::as_str)
}

fn strings(args: &[OsString]) -> Vec<String> {
    args.iter()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect()
}

fn host_triple() -> String {
    let arch = std::env::consts::ARCH;
    match std::env::consts::OS {
        "macos" => format!("{arch}-apple-darwin"),
        "linux" => format!("{arch}-unknown-linux-gnu"),
        "windows" => format!("{arch}-pc-windows-msvc"),
        other => format!("{arch}-unknown-{other}"),
    }
}

fn fixture_help() -> i32 {
    println!(
        "synthetic visibility protocol fixture; invoke through rustup, mise, cargo, or cargo-nextest alias"
    );
    0
}

fn fail(message: String) -> i32 {
    eprintln!("visibility fixture: {message}");
    127
}

fn parse_nextest_signal(value: Option<&str>) -> Result<Option<libc::c_int>, String> {
    match value {
        None => Ok(None),
        Some("9") => Ok(Some(libc::SIGKILL)),
        Some(value) => Err(format!(
            "{NEXTTEST_SIGNAL_ENV} supports only SIGKILL (9), got {value:?}"
        )),
    }
}

fn requested_nextest_signal() -> Result<Option<libc::c_int>, String> {
    match env::var(NEXTTEST_SIGNAL_ENV) {
        Ok(value) => parse_nextest_signal(Some(&value)),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(_)) => Err(format!("{NEXTTEST_SIGNAL_ENV} must be UTF-8")),
    }
}

fn terminate_by_signal(signal: libc::c_int) -> io::Result<()> {
    #[cfg(unix)]
    {
        // SAFETY: getpid has no preconditions; the validated SIGKILL targets this process.
        let pid = unsafe { libc::getpid() };
        // SAFETY: `pid` is this process and the parser permits only SIGKILL.
        if unsafe { libc::kill(pid, signal) } != 0 {
            return Err(io::Error::last_os_error());
        }
        loop {
            std::thread::sleep(std::time::Duration::from_secs(60));
        }
    }
    #[cfg(not(unix))]
    {
        let _ = signal;
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "signal termination is supported only on Unix",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::parse_nextest_signal;

    #[test]
    fn nextest_signal_parser_accepts_only_a_guaranteed_terminating_signal() {
        assert_eq!(parse_nextest_signal(None), Ok(None));
        assert_eq!(parse_nextest_signal(Some("9")), Ok(Some(libc::SIGKILL)));
        for value in ["1", "15", "19", "64", "not-a-signal"] {
            assert!(
                parse_nextest_signal(Some(value)).is_err(),
                "accepted {value:?}"
            );
        }
    }
}
