//! Shared support for Rust black-box tests of the visibility command-line tools.
//!
//! Tests copy the production scripts and their input files into a disposable
//! repository tree, then launch each script as a command-line program. They do
//! not import Python modules or run Python test drivers.

pub mod nextest_result;

use std::env;
#[cfg(unix)]
use std::ffi::CString;
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io;
use std::path::{Component, Path, PathBuf};
#[cfg(unix)]
use std::process::ExitStatus;
#[cfg(unix)]
use std::process::{Child, Command, Stdio};
#[cfg(unix)]
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
#[cfg(unix)]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(unix)]
use std::thread::{self, JoinHandle};
use std::time::Duration;
#[cfg(unix)]
use std::time::Instant;

#[cfg(unix)]
use libc::{self, c_int};
use sha2::{Digest, Sha256};
#[cfg(unix)]
use std::io::{Read, Write};
use tempfile::TempDir;

const DEFAULT_CLI_TIMEOUT: Duration = Duration::from_secs(120);
#[cfg(unix)]
const MAX_CAPTURE_BYTES: usize = 8 * 1024 * 1024;
#[cfg(unix)]
const CHILD_POLL_INTERVAL: Duration = Duration::from_millis(10);
static TEMP_PROBE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// A production visibility tool that a Rust test can launch as a CLI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Tool {
    /// `tools/visibility/status.py`.
    Status,
    /// `tools/visibility/queue.py`.
    Queue,
    /// `tools/visibility/deferred.py`.
    Deferred,
    /// `tools/visibility/subjects.py`.
    Subjects,
}

impl Tool {
    /// Returns the repository-relative source path for this tool.
    pub const fn relative_path(self) -> &'static str {
        match self {
            Self::Status => "tools/visibility/status.py",
            Self::Queue => "tools/visibility/queue.py",
            Self::Deferred => "tools/visibility/deferred.py",
            Self::Subjects => "tools/visibility/subjects.py",
        }
    }
}

/// A file copied byte-for-byte from the current repository source tree.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolCopy {
    /// Destination path under the temporary repository root.
    pub path: PathBuf,
    /// Lowercase SHA-256 of both source and destination bytes.
    pub sha256: String,
    /// Number of copied bytes.
    pub bytes: usize,
}

/// One isolated project tree. The directory is removed when this value drops.
#[derive(Debug)]
pub struct TempRepo {
    _directory: TempDir,
    root: PathBuf,
}

impl TempRepo {
    /// Creates a unique temporary repository root with automatic cleanup.
    pub fn new() -> io::Result<Self> {
        let directory = tempfile::Builder::new()
            .prefix("termrock-visibility-")
            .tempdir()?;
        // macOS commonly exposes the temp directory through `/var`, which is
        // a symlink to `/private/var`. The production queue deliberately opens
        // every path component with `O_NOFOLLOW`, so retain the canonical path.
        let root = directory.path().canonicalize()?;
        Ok(Self {
            _directory: directory,
            root,
        })
    }

    /// Returns the temporary repository root.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Copies one production visibility script into its normal relative path.
    ///
    /// The copied bytes and SHA-256 must both match the source file.
    pub fn copy_tool_exact(&self, tool: Tool) -> io::Result<ToolCopy> {
        self.copy_project_file_exact(Path::new(tool.relative_path()))
    }

    /// Copies one repository file into the same relative path under this root.
    ///
    /// This supports fixed tool inputs such as schemas and JSON registries.
    /// The relative path must contain only normal path components.
    pub fn copy_project_file_exact(&self, relative_path: &Path) -> io::Result<ToolCopy> {
        validate_relative_path(relative_path)?;
        let source = source_repository_root()?.join(relative_path);
        let source_bytes = fs::read(&source)?;
        let destination = self.prepare_destination(relative_path)?;
        match fs::symlink_metadata(&destination) {
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!(
                        "copy destination is not a regular file: {}",
                        relative_path.display()
                    ),
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
        fs::write(&destination, &source_bytes)?;
        let destination_bytes = fs::read(&destination)?;
        if source_bytes != destination_bytes {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "copied file differs from source: {}",
                    relative_path.display()
                ),
            ));
        }
        Ok(ToolCopy {
            path: destination,
            sha256: sha256_hex(&source_bytes),
            bytes: source_bytes.len(),
        })
    }

    /// Writes a fixture file below the temporary root.
    ///
    /// Parent traversal and symlinked destination parents are rejected.
    pub fn write_file(&self, relative_path: &Path, bytes: &[u8]) -> io::Result<PathBuf> {
        validate_relative_path(relative_path)?;
        let destination = self.prepare_destination(relative_path)?;
        match fs::symlink_metadata(&destination) {
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!(
                        "fixture destination is not a regular file: {}",
                        relative_path.display()
                    ),
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
        fs::write(&destination, bytes)?;
        Ok(destination)
    }

    fn prepare_destination(&self, relative_path: &Path) -> io::Result<PathBuf> {
        validate_relative_path(relative_path)?;
        let mut parent = self.root().to_path_buf();
        let components: Vec<_> = relative_path.components().collect();
        for component in &components[..components.len().saturating_sub(1)] {
            let Component::Normal(name) = component else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "invalid fixture path",
                ));
            };
            parent.push(name);
            match fs::symlink_metadata(&parent) {
                Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!(
                            "fixture parent is not a real directory: {}",
                            parent.display()
                        ),
                    ));
                }
                Ok(_) => {}
                Err(error) if error.kind() == io::ErrorKind::NotFound => fs::create_dir(&parent)?,
                Err(error) => return Err(error),
            }
        }
        let Component::Normal(leaf) = components.last().copied().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "fixture path must name a file")
        })?
        else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "fixture path must name a file",
            ));
        };
        parent.push(leaf);
        Ok(parent)
    }
}

/// Copies one production tool through a temporary repository's exact-copy API.
pub fn copy_tool_exact(repo: &TempRepo, tool: Tool) -> io::Result<ToolCopy> {
    repo.copy_tool_exact(tool)
}

/// The result of one child process, with signal termination kept distinct.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CliOutput {
    /// Process exit code, or `None` when the process died from a signal.
    pub exit_code: Option<i32>,
    /// Unix signal number, or `None` when the process exited normally.
    pub signal: Option<i32>,
    /// Retained stdout bytes, up to the capture limit.
    pub stdout: Vec<u8>,
    /// Retained stderr bytes, up to the capture limit.
    pub stderr: Vec<u8>,
    /// True when the process or its output pipes exceeded the configured timeout.
    pub timed_out: bool,
    /// True when stdout exceeded the capture limit. The captured bytes are its prefix.
    pub stdout_truncated: bool,
    /// True when stderr exceeded the capture limit. The captured bytes are its prefix.
    pub stderr_truncated: bool,
}

impl CliOutput {
    /// Returns true only for an ordinary exit status of zero.
    pub fn success(&self) -> bool {
        self.exit_code == Some(0) && self.signal.is_none() && !self.timed_out
    }
}

/// Runs a production Python CLI through `python3`, without a shell or import.
///
/// Environment values override the inherited process environment. An optional
/// byte slice is sent to stdin while stdout and stderr are drained concurrently.
/// On Unix, the command runs in a private process group and is killed and reaped
/// after 120 seconds. No more than 8 MiB of each output stream is retained.
pub fn run_cli(
    script: &Path,
    args: &[String],
    cwd: &Path,
    environment: &[(&str, &str)],
    stdin: Option<&[u8]>,
) -> io::Result<CliOutput> {
    run_cli_with_timeout(script, args, cwd, environment, stdin, DEFAULT_CLI_TIMEOUT)
}

/// Runs a production Python CLI with an explicit wall-clock timeout.
///
/// Output capture remains bounded to 8 MiB for each stream. On Unix, a timeout
/// kills the command's process group, then reaps its direct child, and returns
/// `CliOutput` with `timed_out` set. Non-Unix platforms return
/// `ErrorKind::Unsupported` before spawning a process.
#[cfg(unix)]
pub fn run_cli_with_timeout(
    script: &Path,
    args: &[String],
    cwd: &Path,
    environment: &[(&str, &str)],
    stdin: Option<&[u8]>,
    timeout: Duration,
) -> io::Result<CliOutput> {
    let mut command = Command::new("python3");
    command.arg(script).args(args).current_dir(cwd);
    for (name, value) in environment {
        command.env(name, value);
    }

    run_captured_command(command, stdin, timeout, MAX_CAPTURE_BYTES)
}

/// Non-Unix process-tree containment is not implemented by this test helper.
#[cfg(not(unix))]
pub fn run_cli_with_timeout(
    script: &Path,
    args: &[String],
    cwd: &Path,
    environment: &[(&str, &str)],
    stdin: Option<&[u8]>,
    timeout: Duration,
) -> io::Result<CliOutput> {
    let _ = (script, args, cwd, environment, stdin, timeout);
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "bounded visibility CLI capture requires Unix process groups and nonblocking pipes",
    ))
}

#[cfg(unix)]
#[derive(Debug)]
struct CapturedPipe {
    bytes: Vec<u8>,
    truncated: bool,
}

#[cfg(unix)]
fn run_captured_command(
    mut command: Command,
    stdin: Option<&[u8]>,
    timeout: Duration,
    output_limit: usize,
) -> io::Result<CliOutput> {
    let started = Instant::now();
    let deadline = started.checked_add(timeout).ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "command timeout is too large")
    })?;
    command
        .stdin(if stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }

    let mut child = command.spawn()?;
    let stdout = match child.stdout.take() {
        Some(pipe) => pipe,
        None => {
            terminate_and_reap(&mut child)?;
            return Err(io::Error::other("child stdout pipe was not created"));
        }
    };
    let stderr = match child.stderr.take() {
        Some(pipe) => pipe,
        None => {
            terminate_and_reap(&mut child)?;
            return Err(io::Error::other("child stderr pipe was not created"));
        }
    };
    #[cfg(unix)]
    {
        use std::os::fd::AsRawFd;
        if let Err(error) = set_nonblocking(stdout.as_raw_fd()) {
            let _ = terminate_and_reap(&mut child);
            return Err(error);
        }
        if let Err(error) = set_nonblocking(stderr.as_raw_fd()) {
            let _ = terminate_and_reap(&mut child);
            return Err(error);
        }
    }
    let cancelled = Arc::new(AtomicBool::new(false));

    let stdout_reader = match spawn_pipe_reader(
        stdout,
        output_limit,
        Arc::clone(&cancelled),
        "visibility-cli-stdout",
    ) {
        Ok(reader) => reader,
        Err(error) => {
            cancelled.store(true, Ordering::Release);
            let _ = terminate_and_reap(&mut child);
            return Err(error);
        }
    };
    let stderr_reader = match spawn_pipe_reader(
        stderr,
        output_limit,
        Arc::clone(&cancelled),
        "visibility-cli-stderr",
    ) {
        Ok(reader) => reader,
        Err(error) => {
            cancelled.store(true, Ordering::Release);
            let _ = terminate_and_reap(&mut child);
            let _ = stdout_reader.join();
            return Err(error);
        }
    };
    let stdin_writer = if let Some(input) = stdin {
        let Some(pipe) = child.stdin.take() else {
            cancelled.store(true, Ordering::Release);
            let _ = terminate_and_reap(&mut child);
            let _ = stdout_reader.join();
            let _ = stderr_reader.join();
            return Err(io::Error::other("child stdin pipe was not created"));
        };
        #[cfg(unix)]
        {
            use std::os::fd::AsRawFd;
            if let Err(error) = set_nonblocking(pipe.as_raw_fd()) {
                cancelled.store(true, Ordering::Release);
                let _ = terminate_and_reap(&mut child);
                let _ = stdout_reader.join();
                let _ = stderr_reader.join();
                return Err(error);
            }
        }
        match spawn_stdin_writer(pipe, input.to_vec(), Arc::clone(&cancelled)) {
            Ok(writer) => Some(writer),
            Err(error) => {
                cancelled.store(true, Ordering::Release);
                let _ = terminate_and_reap(&mut child);
                let _ = stdout_reader.join();
                let _ = stderr_reader.join();
                return Err(error);
            }
        }
    } else {
        None
    };

    let mut timed_out = false;
    loop {
        let child_exited = match child_exited_without_reaping(&child) {
            Ok(exited) => exited,
            Err(error) => {
                cancelled.store(true, Ordering::Release);
                let _ = terminate_and_reap(&mut child);
                let _ = stdout_reader.join();
                let _ = stderr_reader.join();
                if let Some(writer) = stdin_writer {
                    let _ = writer.join();
                }
                return Err(error);
            }
        };

        let pipes_finished = stdout_reader.is_finished()
            && stderr_reader.is_finished()
            && stdin_writer.as_ref().is_none_or(JoinHandle::is_finished);
        if child_exited && pipes_finished {
            // The child remains an unreaped zombie until wait(), reserving its
            // process-group ID through this final group cleanup.
            terminate_process_group(child.id());
            return collect_capture(
                child.wait(),
                stdout_reader,
                stderr_reader,
                stdin_writer,
                timed_out,
            );
        }

        if Instant::now() >= deadline {
            timed_out = true;
            cancelled.store(true, Ordering::Release);
            // `child_exited_without_reaping` deliberately leaves the leader
            // waitable. Signal its group before reaping so the numeric PGID
            // cannot be reused for an unrelated process group.
            terminate_process_group(child.id());
            let status = child.wait();
            return collect_capture(
                status,
                stdout_reader,
                stderr_reader,
                stdin_writer,
                timed_out,
            );
        }
        thread::sleep(CHILD_POLL_INTERVAL.min(deadline.saturating_duration_since(Instant::now())));
    }
}

#[cfg(unix)]
fn spawn_pipe_reader<R>(
    pipe: R,
    output_limit: usize,
    cancelled: Arc<AtomicBool>,
    name: &'static str,
) -> io::Result<JoinHandle<io::Result<CapturedPipe>>>
where
    R: Read + Send + 'static,
{
    thread::Builder::new()
        .name(name.to_owned())
        .spawn(move || capture_pipe(pipe, output_limit, cancelled))
}

#[cfg(unix)]
fn capture_pipe(
    mut pipe: impl Read,
    output_limit: usize,
    cancelled: Arc<AtomicBool>,
) -> io::Result<CapturedPipe> {
    let mut bytes = Vec::with_capacity(output_limit.min(16 * 1024));
    let mut buffer = [0_u8; 16 * 1024];
    let mut truncated = false;
    loop {
        if cancelled.load(Ordering::Acquire) {
            break;
        }
        let count = match pipe.read(&mut buffer) {
            Ok(count) => count,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                thread::sleep(CHILD_POLL_INTERVAL);
                continue;
            }
            Err(error) => return Err(error),
        };
        if count == 0 {
            break;
        }
        let remaining = output_limit.saturating_sub(bytes.len());
        let retained = count.min(remaining);
        bytes.extend_from_slice(&buffer[..retained]);
        truncated |= retained < count;
    }
    Ok(CapturedPipe { bytes, truncated })
}

#[cfg(unix)]
fn spawn_stdin_writer(
    mut pipe: std::process::ChildStdin,
    input: Vec<u8>,
    cancelled: Arc<AtomicBool>,
) -> io::Result<JoinHandle<io::Result<()>>> {
    thread::Builder::new()
        .name("visibility-cli-stdin".to_owned())
        .spawn(move || {
            let mut written = 0;
            while written < input.len() {
                if cancelled.load(Ordering::Acquire) {
                    break;
                }
                match pipe.write(&input[written..]) {
                    Ok(0) => {
                        return Err(io::Error::new(
                            io::ErrorKind::WriteZero,
                            "stdin pipe closed",
                        ));
                    }
                    Ok(count) => written += count,
                    Err(error) if error.kind() == io::ErrorKind::BrokenPipe => return Ok(()),
                    Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                    Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                        thread::sleep(CHILD_POLL_INTERVAL);
                    }
                    Err(error) => return Err(error),
                }
            }
            Ok(())
        })
}

#[cfg(unix)]
fn set_nonblocking(fd: libc::c_int) -> io::Result<()> {
    // SAFETY: fcntl only inspects and updates flags for this open pipe descriptor.
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: `fd` remains open and the command accepts a bitmask of file status flags.
    if unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

#[cfg(unix)]
fn join_pipe_reader(reader: JoinHandle<io::Result<CapturedPipe>>) -> io::Result<CapturedPipe> {
    reader
        .join()
        .map_err(|_| io::Error::other("output capture thread panicked"))?
}

#[cfg(unix)]
fn join_stdin_writer(writer: JoinHandle<io::Result<()>>) -> io::Result<()> {
    writer
        .join()
        .map_err(|_| io::Error::other("stdin writer thread panicked"))?
}

#[cfg(unix)]
fn collect_capture(
    status: io::Result<ExitStatus>,
    stdout_reader: JoinHandle<io::Result<CapturedPipe>>,
    stderr_reader: JoinHandle<io::Result<CapturedPipe>>,
    stdin_writer: Option<JoinHandle<io::Result<()>>>,
    timed_out: bool,
) -> io::Result<CliOutput> {
    let stdout = join_pipe_reader(stdout_reader);
    let stderr = join_pipe_reader(stderr_reader);
    let stdin = stdin_writer.map(join_stdin_writer).transpose();
    let status = status?;
    let stdout = stdout?;
    let stderr = stderr?;
    stdin?;
    Ok(CliOutput {
        exit_code: status.code(),
        signal: exit_signal(&status),
        stdout: stdout.bytes,
        stderr: stderr.bytes,
        timed_out,
        stdout_truncated: stdout.truncated,
        stderr_truncated: stderr.truncated,
    })
}

#[cfg(unix)]
fn child_exited_without_reaping(child: &Child) -> io::Result<bool> {
    loop {
        let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
        // SAFETY: `child` is this process's live, unreaped child and `info` is
        // writable storage for waitid's result. WNOWAIT preserves its PID.
        let result = unsafe {
            libc::waitid(
                libc::P_PID,
                child.id() as libc::id_t,
                &mut info,
                libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
            )
        };
        if result == 0 {
            // SAFETY: waitid initialized the siginfo result for this call.
            let observed_pid = unsafe { info.si_pid() };
            if observed_pid == 0 {
                return Ok(false);
            }
            if observed_pid == child.id() as libc::pid_t {
                return Ok(true);
            }
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "waitid returned a different child process",
            ));
        }
        let error = io::Error::last_os_error();
        if error.kind() == io::ErrorKind::Interrupted {
            continue;
        }
        return Err(error);
    }
}

#[cfg(unix)]
fn terminate_and_reap(child: &mut Child) -> io::Result<ExitStatus> {
    terminate_process_group(child.id());
    let _ = child.kill();
    child.wait()
}

#[cfg(unix)]
fn terminate_process_group(child_id: u32) {
    let process_group = -(child_id as libc::pid_t);
    // SAFETY: the child is still unreaped and owns this private process group.
    let _ = unsafe { libc::kill(process_group, libc::SIGKILL) };
}

/// Creates command-name links to the compiled protocol fixture binary.
///
/// Integration tests pass `Path::new(env!("CARGO_BIN_EXE_fixture"))` as the
/// executable. The returned value is a PATH with `bin_dir` prepended.
pub fn install_fixture_aliases(
    executable: &Path,
    bin_dir: &Path,
    aliases: &[&str],
) -> io::Result<OsString> {
    if !executable.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "compiled fixture binary does not exist: {}",
                executable.display()
            ),
        ));
    }
    fs::create_dir_all(bin_dir)?;
    for alias in aliases {
        if alias.is_empty()
            || alias.contains('/')
            || alias.contains('\\')
            || *alias == "."
            || *alias == ".."
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid fixture command alias",
            ));
        }
        let destination = bin_dir.join(alias);
        if fs::symlink_metadata(&destination).is_ok() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!(
                    "fixture command alias already exists: {}",
                    destination.display()
                ),
            ));
        }
        install_alias(executable, &destination)?;
    }
    prepend_path(bin_dir)
}

/// Prepends one directory to the inherited PATH and returns the joined value.
pub fn prepend_path(directory: &Path) -> io::Result<OsString> {
    let mut entries = vec![directory.to_path_buf()];
    if let Some(path) = env::var_os("PATH") {
        entries.extend(env::split_paths(&path));
    }
    env::join_paths(entries).map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))
}

/// Holds the exact advisory lock used by the visibility queue for a task file.
///
/// This lets a Rust CLI test control a real queue-lock interleaving. Dropping
/// the returned guard closes the file and releases the lock.
#[derive(Debug)]
pub struct QueueLockGuard {
    _file: File,
}

#[cfg(unix)]
impl Drop for QueueLockGuard {
    fn drop(&mut self) {
        // Match the production Python tool's `fcntl.flock(fd, LOCK_UN)`.
        let _ = flock_file(&self._file, libc::LOCK_UN);
    }
}

/// Acquires the queue's process-shared lock for `tasks_path`.
#[cfg(unix)]
pub fn hold_queue_lock(tasks_path: &Path) -> io::Result<QueueLockGuard> {
    let temp_root = python_temp_root()?;
    let file = open_queue_lock_file(tasks_path, &temp_root)?;
    flock_file(&file, libc::LOCK_EX)?;
    Ok(QueueLockGuard { _file: file })
}

/// Queue locking is available only on Unix, like the production fcntl lock.
#[cfg(not(unix))]
pub fn hold_queue_lock(_tasks_path: &Path) -> io::Result<QueueLockGuard> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "the queue lock fixture requires Unix flock behavior",
    ))
}

/// Returns the repository root that contains this standalone crate.
pub fn source_repository_root() -> io::Result<PathBuf> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
}

/// SHA-256 hex encoding used for exact copied-file identities and queue locks.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(output, "{byte:02x}");
    }
    output
}

fn absolute_path_like_python(path: &Path) -> io::Result<PathBuf> {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "queue lock path must not contain parent traversal",
                ));
            }
            Component::CurDir => {}
            component => normalized.push(component.as_os_str()),
        }
    }
    if normalized.is_absolute() {
        Ok(normalized)
    } else {
        Ok(env::current_dir()?.join(normalized))
    }
}

fn python_temp_root() -> io::Result<PathBuf> {
    python_temp_root_from_candidates(
        env::var_os("TMPDIR"),
        env::var_os("TEMP"),
        env::var_os("TMP"),
    )
}

fn python_temp_root_from_candidates(
    tmpdir: Option<OsString>,
    temp: Option<OsString>,
    tmp: Option<OsString>,
) -> io::Result<PathBuf> {
    select_python_temp_root(python_tempdir_candidates(tmpdir, temp, tmp))
}

fn python_tempdir_candidates(
    tmpdir: Option<OsString>,
    temp: Option<OsString>,
    tmp: Option<OsString>,
) -> Vec<PathBuf> {
    let mut candidates = [tmpdir, temp, tmp]
        .into_iter()
        .flatten()
        .map(PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty())
        .collect::<Vec<_>>();
    #[cfg(unix)]
    candidates.extend([
        PathBuf::from("/tmp"),
        PathBuf::from("/var/tmp"),
        PathBuf::from("/usr/tmp"),
    ]);
    #[cfg(windows)]
    if let Some(profile) = env::var_os("USERPROFILE") {
        candidates.push(PathBuf::from(profile).join("AppData/Local/Temp"));
    }
    // Python tempfile tries the current working directory after its platform
    // defaults when none of the earlier candidates can hold a probe file.
    if let Ok(current_dir) = env::current_dir() {
        candidates.push(current_dir);
    }
    candidates
}

fn select_python_temp_root(candidates: impl IntoIterator<Item = PathBuf>) -> io::Result<PathBuf> {
    for candidate in candidates {
        if !is_writable_temp_directory(&candidate) {
            continue;
        }
        if let Ok(canonical) = candidate.canonicalize() {
            return Ok(canonical);
        }
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "no writable temporary directory matches Python tempfile selection",
    ))
}

fn is_writable_temp_directory(directory: &Path) -> bool {
    if !directory.is_dir() {
        return false;
    }
    for _ in 0..100 {
        let probe = directory.join(format!(
            ".termrock-visibility-temp-probe-{}-{}",
            std::process::id(),
            TEMP_PROBE_SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        match OpenOptions::new().write(true).create_new(true).open(&probe) {
            Ok(file) => {
                drop(file);
                return fs::remove_file(probe).is_ok();
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(_) => return false,
        }
    }
    false
}

/// Environment names consumed by the compiled synthetic protocol fixture.
pub mod fixture_env {
    /// File containing exact Nextest list JSON bytes.
    pub const NEXTTEST_LIST: &str = "VISIBILITY_FIXTURE_NEXTTEST_LIST";
    /// Exit code for a Nextest list command. Defaults to zero.
    pub const NEXTTEST_LIST_EXIT: &str = "VISIBILITY_FIXTURE_NEXTTEST_LIST_EXIT";
    /// File containing exact Nextest run JSONL bytes.
    pub const NEXTTEST_RUN: &str = "VISIBILITY_FIXTURE_NEXTTEST_RUN";
    /// Exit code for a Nextest run command. Defaults to zero.
    pub const NEXTTEST_EXIT: &str = "VISIBILITY_FIXTURE_NEXTTEST_EXIT";
    /// Optional Unix signal that terminates the synthetic Nextest run process.
    /// Only signal 9 (SIGKILL) is accepted.
    pub const NEXTTEST_SIGNAL: &str = "VISIBILITY_FIXTURE_NEXTTEST_SIGNAL";
    /// Fake cargo-nextest version text.
    pub const NEXTEST_VERSION: &str = "VISIBILITY_FIXTURE_NEXTEST_VERSION";
    /// Nonzero exit code for fake cargo-nextest version lookup.
    pub const NEXTEST_VERSION_EXIT: &str = "VISIBILITY_FIXTURE_NEXTEST_VERSION_EXIT";
    /// File receiving one fixture invocation record per line.
    pub const TRACE: &str = "VISIBILITY_FIXTURE_TRACE";
    /// File containing Cargo metadata JSON; strings may use documented path tokens.
    pub const CARGO_METADATA: &str = "VISIBILITY_FIXTURE_CARGO_METADATA";
    /// File containing raw Cargo metadata stdout, bypassing JSON path substitution.
    pub const CARGO_METADATA_RAW: &str = "VISIBILITY_FIXTURE_CARGO_METADATA_RAW";
    /// Nonzero exit code for fake Cargo metadata.
    pub const CARGO_METADATA_EXIT: &str = "VISIBILITY_FIXTURE_CARGO_METADATA_EXIT";
    /// File containing fake Cargo metadata stderr bytes.
    pub const CARGO_METADATA_STDERR: &str = "VISIBILITY_FIXTURE_CARGO_METADATA_STDERR";
    /// Nonzero exit code for fake Cargo build.
    pub const CARGO_BUILD_EXIT: &str = "VISIBILITY_FIXTURE_CARGO_BUILD_EXIT";
    /// File containing raw Cargo build stdout, bypassing artifact generation.
    pub const CARGO_BUILD_STDOUT: &str = "VISIBILITY_FIXTURE_CARGO_BUILD_STDOUT";
    /// File containing fake Cargo build stderr bytes.
    pub const CARGO_BUILD_STDERR: &str = "VISIBILITY_FIXTURE_CARGO_BUILD_STDERR";
    /// Artifact probe mode: valid, missing, malformed, duplicate, wrong-package,
    /// wrong-target, fresh, test-profile, outside-target, or non-executable.
    pub const ARTIFACT_MODE: &str = "VISIBILITY_FIXTURE_ARTIFACT_MODE";
    /// Fake Cargo version text.
    pub const CARGO_VERSION: &str = "VISIBILITY_FIXTURE_CARGO_VERSION";
    /// Nonzero exit code for fake Cargo version lookup.
    pub const CARGO_VERSION_EXIT: &str = "VISIBILITY_FIXTURE_CARGO_VERSION_EXIT";
    /// Fake rustc version text.
    pub const RUSTC_VERSION: &str = "VISIBILITY_FIXTURE_RUSTC_VERSION";
    /// Nonzero exit code for fake rustc version lookup.
    pub const RUSTC_VERSION_EXIT: &str = "VISIBILITY_FIXTURE_RUSTC_VERSION_EXIT";
    /// Fake `rustc -Vv` text.
    pub const RUSTC_VERBOSE: &str = "VISIBILITY_FIXTURE_RUSTC_VERBOSE";
    /// Nonzero exit code for fake `rustc -Vv` lookup.
    pub const RUSTC_VERBOSE_EXIT: &str = "VISIBILITY_FIXTURE_RUSTC_VERBOSE_EXIT";
    /// Fake `rustup show home` output.
    pub const RUSTUP_HOME: &str = "VISIBILITY_FIXTURE_RUSTUP_HOME";
}

fn validate_relative_path(path: &Path) -> io::Result<()> {
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "fixture path must contain only normal relative components",
        ));
    }
    Ok(())
}

#[cfg(unix)]
fn exit_signal(status: &std::process::ExitStatus) -> Option<i32> {
    use std::os::unix::process::ExitStatusExt;
    status.signal()
}

#[cfg(unix)]
fn effective_uid() -> io::Result<u32> {
    // SAFETY: geteuid has no preconditions and returns the current process UID.
    Ok(unsafe { libc::geteuid() })
}

#[cfg(unix)]
fn flock_file(file: &File, operation: c_int) -> io::Result<()> {
    use std::os::fd::AsRawFd;

    // SAFETY: `file` keeps this valid descriptor open for the duration of the call.
    let result = unsafe { libc::flock(file.as_raw_fd(), operation) };
    if result == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

#[cfg(unix)]
fn open_queue_lock_file(tasks_path: &Path, temp_root: &Path) -> io::Result<File> {
    let absolute_path = absolute_path_like_python(tasks_path)?;
    let lock_key = sha256_hex(
        absolute_path
            .to_str()
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "queue path is not valid UTF-8")
            })?
            .as_bytes(),
    );
    let temp_directory = open_directory_chain_no_follow(temp_root)?;
    let directory_name = format!("termrock-visibility-queue-{}", effective_uid()?);
    let lock_directory = open_private_lock_directory_at(&temp_directory, &directory_name)?;
    open_private_lock_file_at(&lock_directory, &format!("{lock_key}.lock"))
}

#[cfg(unix)]
fn open_directory_chain_no_follow(path: &Path) -> io::Result<File> {
    use std::os::fd::{AsRawFd, FromRawFd};
    use std::os::unix::ffi::OsStrExt;

    if !path.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "temporary lock root must be absolute",
        ));
    }
    let root_name = CString::new("/").expect("root path has no NUL byte");
    let flags = libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC;
    // SAFETY: `root_name` is a valid NUL-terminated path and flags request a
    // directory descriptor without following a symlink.
    let root_fd = unsafe { libc::open(root_name.as_ptr(), flags) };
    if root_fd < 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: `root_fd` is a newly opened descriptor now owned by this File.
    let mut directory = unsafe { File::from_raw_fd(root_fd) };
    for component in path.components() {
        let Component::Normal(name) = component else {
            if matches!(component, Component::RootDir) {
                continue;
            }
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "temporary lock root has an invalid path component",
            ));
        };
        let name = CString::new(name.as_bytes()).map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidInput, "path component contains NUL")
        })?;
        // SAFETY: the parent descriptor is open and name is a single path
        // component. O_NOFOLLOW and O_DIRECTORY reject symlink/non-directory
        // ancestors at every step.
        let child_fd = unsafe { libc::openat(directory.as_raw_fd(), name.as_ptr(), flags) };
        if child_fd < 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: `child_fd` is a newly opened descriptor now owned by this File.
        let child_directory = unsafe { File::from_raw_fd(child_fd) };
        directory = child_directory;
    }
    Ok(directory)
}

#[cfg(unix)]
fn open_private_lock_directory_at(parent: &File, name: &str) -> io::Result<File> {
    use std::os::fd::{AsRawFd, FromRawFd};
    use std::os::unix::fs::{MetadataExt, PermissionsExt};

    let name = CString::new(name)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "lock directory name has NUL"))?;
    // SAFETY: parent is a live directory descriptor and name is one component.
    if unsafe { libc::mkdirat(parent.as_raw_fd(), name.as_ptr(), 0o700) } != 0 {
        let error = io::Error::last_os_error();
        if error.kind() != io::ErrorKind::AlreadyExists {
            return Err(error);
        }
    }
    let flags = libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC;
    // SAFETY: parent is open; O_NOFOLLOW and O_DIRECTORY reject symlinks.
    let directory_fd = unsafe { libc::openat(parent.as_raw_fd(), name.as_ptr(), flags) };
    if directory_fd < 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: `directory_fd` is now owned by this File.
    let directory = unsafe { File::from_raw_fd(directory_fd) };
    let metadata = directory.metadata()?;
    if !metadata.is_dir()
        || metadata.uid() != effective_uid()?
        || metadata.permissions().mode() & 0o077 != 0
    {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "queue lock directory is not a private directory owned by this user",
        ));
    }
    Ok(directory)
}

#[cfg(unix)]
fn open_private_lock_file_at(parent: &File, name: &str) -> io::Result<File> {
    use std::os::fd::{AsRawFd, FromRawFd};
    use std::os::unix::fs::{MetadataExt, PermissionsExt};

    let name = CString::new(name)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "lock file name has NUL"))?;
    let flags = libc::O_RDWR | libc::O_CREAT | libc::O_NOFOLLOW | libc::O_CLOEXEC;
    // SAFETY: parent is an open checked directory; O_NOFOLLOW protects the
    // final lock-file component from symlink substitution.
    let file_fd = unsafe { libc::openat(parent.as_raw_fd(), name.as_ptr(), flags, 0o600) };
    if file_fd < 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: `file_fd` is now owned by this File.
    let file = unsafe { File::from_raw_fd(file_fd) };
    let metadata = file.metadata()?;
    if !metadata.is_file()
        || metadata.uid() != effective_uid()?
        || metadata.permissions().mode() & 0o077 != 0
    {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "queue lock file is not a private regular file owned by this user",
        ));
    }
    Ok(file)
}

#[cfg(unix)]
fn install_alias(executable: &Path, destination: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(executable, destination)
}

#[cfg(not(unix))]
fn install_alias(executable: &Path, destination: &Path) -> io::Result<()> {
    fs::copy(executable, destination).map(|_| ())
}

#[cfg(test)]
mod tests {
    #[cfg(unix)]
    use super::run_cli_with_timeout;
    use super::{TempRepo, Tool, source_repository_root};
    #[cfg(unix)]
    use std::env;
    use std::fs;
    use std::io;
    #[cfg(unix)]
    use std::io::Write;
    use std::path::Path;
    #[cfg(unix)]
    use std::path::PathBuf;
    #[cfg(unix)]
    use std::process::Command;
    #[cfg(unix)]
    use std::time::{Duration, Instant};

    #[test]
    fn temp_repo_is_unique_and_copy_keeps_the_production_tool_path_and_bytes() -> io::Result<()> {
        let first = TempRepo::new()?;
        let second = TempRepo::new()?;
        assert_ne!(first.root(), second.root());
        assert_eq!(first.root().canonicalize()?, first.root());

        let copied = first.copy_tool_exact(Tool::Queue)?;
        assert_eq!(copied.path, first.root().join("tools/visibility/queue.py"));
        assert_eq!(copied.bytes, std::fs::read(&copied.path)?.len());
        assert_eq!(
            std::fs::read(&copied.path)?,
            std::fs::read(source_repository_root()?.join(Tool::Queue.relative_path()))?
        );
        assert_eq!(copied.sha256.len(), 64);
        Ok(())
    }

    #[test]
    fn fixture_writer_rejects_parent_traversal() -> io::Result<()> {
        let repo = TempRepo::new()?;
        let error = repo.write_file(Path::new("../outside"), b"no").unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn queue_lock_temp_root_uses_python_fallback_when_temp_variables_are_clear() -> io::Result<()> {
        let selected = super::python_temp_root_from_candidates(None, None, None)?;
        assert_eq!(selected, Path::new("/tmp").canonicalize()?);
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn queue_lock_temp_root_uses_python_temp_variable_precedence() {
        let candidates = super::python_tempdir_candidates(
            Some("/first".into()),
            Some("/second".into()),
            Some("/third".into()),
        );
        assert_eq!(
            &candidates[..6],
            &[
                PathBuf::from("/first"),
                PathBuf::from("/second"),
                PathBuf::from("/third"),
                PathBuf::from("/tmp"),
                PathBuf::from("/var/tmp"),
                PathBuf::from("/usr/tmp"),
            ]
        );
    }

    #[cfg(unix)]
    #[test]
    fn queue_lock_temp_root_tries_the_current_directory_last() -> io::Result<()> {
        let directory = TempRepo::new()?;
        let selected = super::select_python_temp_root([directory.root().to_path_buf()])?;
        assert_eq!(selected, directory.root());
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn queue_lock_rejects_a_symlink_at_the_lock_file_path() -> io::Result<()> {
        use std::os::unix::fs::{PermissionsExt, symlink};

        let repo = TempRepo::new()?;
        let tasks_path = repo.write_file(
            Path::new("docs/implementation/visibility/tasks.json"),
            b"{}\n",
        )?;
        let absolute_path = super::absolute_path_like_python(&tasks_path)?;
        let lock_key = super::sha256_hex(absolute_path.to_str().unwrap().as_bytes());
        let lock_dir_name = format!("termrock-visibility-queue-{}", super::effective_uid()?);
        let temp_directory = super::open_directory_chain_no_follow(repo.root())?;
        let _lock_directory =
            super::open_private_lock_directory_at(&temp_directory, &lock_dir_name)?;
        let lock_path = repo
            .root()
            .join(lock_dir_name)
            .join(format!("{lock_key}.lock"));
        let target = repo.write_file(Path::new("outside.lock"), b"keep unchanged")?;
        fs::set_permissions(&target, fs::Permissions::from_mode(0o600))?;
        symlink(&target, &lock_path)?;

        let result = super::open_queue_lock_file(&tasks_path, repo.root());
        fs::remove_file(&lock_path)?;
        assert!(result.is_err(), "lock helper followed a symlink lock file");
        assert_eq!(fs::read(target)?, b"keep unchanged");
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn queue_lock_rejects_a_symlink_at_the_lock_directory_path() -> io::Result<()> {
        use std::os::unix::fs::symlink;

        let repo = TempRepo::new()?;
        let tasks_path = repo.write_file(
            Path::new("docs/implementation/visibility/tasks.json"),
            b"{}\n",
        )?;
        let lock_dir_name = format!("termrock-visibility-queue-{}", super::effective_uid()?);
        let target = repo.root().join("target-directory");
        fs::create_dir(&target)?;
        symlink(&target, repo.root().join(lock_dir_name))?;

        let result = super::open_queue_lock_file(&tasks_path, repo.root());
        assert!(
            result.is_err(),
            "lock helper followed a symlink lock directory"
        );
        assert_eq!(fs::read_dir(target)?.count(), 0);
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn queue_lock_rejects_a_symlink_in_the_temp_root_path() -> io::Result<()> {
        use std::os::unix::fs::symlink;

        let repo = TempRepo::new()?;
        let elsewhere = TempRepo::new()?;
        let tasks_path = repo.write_file(
            Path::new("docs/implementation/visibility/tasks.json"),
            b"{}\n",
        )?;
        let alias = repo.root().join("temp-alias");
        symlink(elsewhere.root(), &alias)?;

        let result = super::open_queue_lock_file(&tasks_path, &alias);
        assert!(result.is_err(), "lock helper followed a temp-root symlink");
        assert_eq!(fs::read_dir(elsewhere.root())?.count(), 0);
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn command_runner_drains_output_while_writing_input_and_caps_capture() -> io::Result<()> {
        let input = vec![b'x'; 1024 * 1024];
        let command = Command::new("/bin/cat");
        let output =
            super::run_captured_command(command, Some(&input), Duration::from_secs(5), 4096)?;

        assert_eq!(output.exit_code, Some(0));
        assert_eq!(output.signal, None);
        assert!(!output.timed_out);
        assert_eq!(output.stdout, vec![b'x'; 4096]);
        assert!(output.stdout_truncated);
        assert!(output.stderr.is_empty());
        assert!(!output.stderr_truncated);
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn command_runner_kills_and_reaps_a_timed_out_child() -> io::Result<()> {
        let mut command = Command::new("/bin/sleep");
        command.arg("30");
        let output = super::run_captured_command(command, None, Duration::from_millis(50), 1024)?;

        assert!(output.timed_out);
        assert_eq!(output.exit_code, None);
        assert_eq!(output.signal, Some(libc::SIGKILL));
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn waitid_observes_exit_without_reaping_the_child() -> io::Result<()> {
        use std::os::unix::process::CommandExt;

        let mut command = Command::new("/bin/sleep");
        command.arg("0.02").process_group(0);
        let mut child = command.spawn()?;
        let deadline = Instant::now() + Duration::from_secs(2);
        while !super::child_exited_without_reaping(&child)? {
            assert!(
                Instant::now() < deadline,
                "child did not exit before the deadline"
            );
            std::thread::sleep(Duration::from_millis(2));
        }

        assert!(super::child_exited_without_reaping(&child)?);
        assert!(child.wait()?.success());
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn command_runner_stops_reading_when_a_detached_descendant_keeps_writing() -> io::Result<()> {
        let repo = TempRepo::new()?;
        let ready_path = repo.root().join("detached-writer-ready");
        let mut command = Command::new(env::current_exe()?);
        command
            .args(["--exact", "tests::runner_process_probe", "--nocapture"])
            .env("TERMROCK_VISIBILITY_RUNNER_PROBE", "leader")
            .env("TERMROCK_VISIBILITY_RUNNER_READY", &ready_path);
        let started = Instant::now();
        let output = super::run_captured_command(command, None, Duration::from_secs(2), 1024)?;

        assert!(output.timed_out);
        assert_eq!(output.exit_code, Some(0));
        assert_eq!(fs::read(&ready_path)?, b"writer started");
        assert!(
            started.elapsed() < Duration::from_secs(4),
            "runner blocked joining a reader after timeout"
        );
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn runner_process_probe() {
        use std::os::unix::process::CommandExt;

        match env::var("TERMROCK_VISIBILITY_RUNNER_PROBE").as_deref() {
            Ok("leader") => {
                let executable = env::current_exe().expect("current test executable");
                let mut command = Command::new(executable);
                command
                    .args(["--exact", "tests::runner_process_probe", "--nocapture"])
                    .env("TERMROCK_VISIBILITY_RUNNER_PROBE", "writer")
                    .process_group(0);
                let writer = command.spawn().expect("spawn detached pipe writer");
                let ready_path = env::var_os("TERMROCK_VISIBILITY_RUNNER_READY")
                    .expect("runner ready marker path");
                let ready_path = PathBuf::from(ready_path);
                let ready_deadline = Instant::now() + Duration::from_secs(1);
                while !ready_path.is_file() {
                    assert!(
                        Instant::now() < ready_deadline,
                        "detached writer did not start"
                    );
                    std::thread::sleep(Duration::from_millis(2));
                }
                println!("detached writer pid={}", writer.id());
                io::stdout().flush().expect("flush leader output");
                drop(writer);
            }
            Ok("writer") => {
                let ready_path = env::var_os("TERMROCK_VISIBILITY_RUNNER_READY")
                    .expect("runner ready marker path");
                fs::write(ready_path, b"writer started").expect("write runner ready marker");
                let mut stdout = io::stdout().lock();
                let bytes = [b'x'; 8 * 1024];
                let end = Instant::now() + Duration::from_secs(10);
                while Instant::now() < end {
                    if stdout.write_all(&bytes).is_err() || stdout.flush().is_err() {
                        break;
                    }
                }
            }
            Ok(other) => panic!("unknown runner probe mode: {other}"),
            Err(env::VarError::NotPresent) => {}
            Err(error) => panic!("invalid runner probe mode: {error}"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn public_cli_runner_executes_the_copied_production_tool() -> io::Result<()> {
        let repo = TempRepo::new()?;
        let copied = repo.copy_tool_exact(Tool::Queue)?;
        let output = run_cli_with_timeout(
            &copied.path,
            &["--help".to_owned()],
            repo.root(),
            &[],
            None,
            Duration::from_secs(5),
        )?;

        assert!(
            output.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("usage:"));
        Ok(())
    }
}
