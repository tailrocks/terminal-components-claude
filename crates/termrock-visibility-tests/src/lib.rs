pub mod nextest_result;

use std::io;
#[cfg(unix)]
use std::process::ExitStatus;
#[cfg(unix)]
use std::process::{Child, Command, Stdio};
#[cfg(unix)]
use std::sync::Arc;
#[cfg(unix)]
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
#[cfg(unix)]
use std::thread::{self, JoinHandle};
use std::time::Duration;
#[cfg(unix)]
use std::time::Instant;

use sha2::{Digest, Sha256};
#[cfg(unix)]
use std::io::{Read, Write};

#[cfg(unix)]
const MAX_CAPTURE_BYTES: usize = 8 * 1024 * 1024;
#[cfg(unix)]
const CHILD_POLL_INTERVAL: Duration = Duration::from_millis(10);

#[cfg(unix)]
fn deadline_has_elapsed(now: Instant, deadline: Instant) -> bool {
    now >= deadline
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CliOutput {
    pub exit_code: Option<i32>,
    pub signal: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub timed_out: bool,
    pub stdout_truncated: bool,
    pub stderr_truncated: bool,
}

#[cfg(unix)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapturedCommand {
    pub output: CliOutput,
    pub cancellation_signal: Option<i32>,
}

#[cfg(unix)]
#[derive(Debug)]
struct CapturedPipe {
    bytes: Vec<u8>,
    truncated: bool,
}

#[cfg(unix)]
pub fn run_command_with_cancellation(
    command: Command,
    timeout: Duration,
    cancellation_signal: Option<&AtomicI32>,
) -> io::Result<CapturedCommand> {
    run_captured_command_with_cancel(
        command,
        None,
        timeout,
        MAX_CAPTURE_BYTES,
        cancellation_signal,
    )
}

#[cfg(unix)]
fn run_captured_command_with_cancel(
    mut command: Command,
    stdin: Option<&[u8]>,
    timeout: Duration,
    output_limit: usize,
    cancellation_signal: Option<&AtomicI32>,
) -> io::Result<CapturedCommand> {
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
        let requested_signal = cancellation_signal.and_then(|signal| {
            let value = signal.load(Ordering::Acquire);
            (value > 0).then_some(value)
        });
        if let Some(signal) = requested_signal {
            cancelled.store(true, Ordering::Release);

            terminate_process_group(child.id());
            let output = collect_capture(
                child.wait(),
                stdout_reader,
                stderr_reader,
                stdin_writer,
                false,
            )?;
            return Ok(CapturedCommand {
                output,
                cancellation_signal: Some(signal),
            });
        }

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

        let observed_at = Instant::now();
        if deadline_has_elapsed(observed_at, deadline) {
            timed_out = true;
            cancelled.store(true, Ordering::Release);

            terminate_process_group(child.id());
            let status = child.wait();
            return collect_capture(
                status,
                stdout_reader,
                stderr_reader,
                stdin_writer,
                timed_out,
            )
            .map(|output| CapturedCommand {
                output,
                cancellation_signal: None,
            });
        }

        if child_exited && pipes_finished {
            terminate_process_group(child.id());
            return collect_capture(
                child.wait(),
                stdout_reader,
                stderr_reader,
                stdin_writer,
                timed_out,
            )
            .map(|output| CapturedCommand {
                output,
                cancellation_signal: None,
            });
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
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 {
        return Err(io::Error::last_os_error());
    }

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

        let result = unsafe {
            libc::waitid(
                libc::P_PID,
                child.id() as libc::id_t,
                &mut info,
                libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
            )
        };
        if result == 0 {
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

    let _ = unsafe { libc::kill(process_group, libc::SIGKILL) };
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(output, "{byte:02x}");
    }
    output
}

#[cfg(unix)]
fn exit_signal(status: &std::process::ExitStatus) -> Option<i32> {
    use std::os::unix::process::ExitStatusExt;
    status.signal()
}
