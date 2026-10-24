//! Keep a second ESC when one read contains Esc and then Alt+key.
//!
//! Crossterm 0.29 parses `\x1b\x1b` as one Esc and clears its buffer, so the
//! Alt prefix is gone and the next byte is a plain character. TablePro's
//! explain-analyze chord is Esc, then Alt+x. A slow 160-column frame delivers
//! those bytes in one read (`\x1b\x1bx`). This bridge owns the real tty and
//! writes the first Esc to a side pty before the second Esc, so Crossterm
//! sees two reads.

use std::io::{self, ErrorKind};
use std::ops::Range;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use ratatui_crossterm::crossterm::event::poll as crossterm_poll;
use termrock_core::keys::keyboard::{QUERY, parse_flags_reply};

/// Index of the second ESC in the first `\x1b\x1b` pair.
pub(crate) fn second_esc_index(bytes: &[u8]) -> Option<usize> {
    bytes
        .windows(2)
        .position(|pair| pair == [0x1b, 0x1b])
        .map(|index| index + 1)
}

struct Bridge {
    real: OwnedFd,
    master: OwnedFd,
    stash: Vec<u8>,
    /// Termios of the real tty before this bridge switched it to raw mode.
    original: libc::termios,
}

static BRIDGE: Mutex<Option<Bridge>> = Mutex::new(None);

struct RestoreOnFailure<'fd> {
    real: &'fd OwnedFd,
    original: libc::termios,
    stdin_touched: bool,
    armed: bool,
}

impl Drop for RestoreOnFailure<'_> {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        if self.stdin_touched {
            // SAFETY: `real` is an owned duplicate of the tty stdin captured by install.
            unsafe { libc::dup2(self.real.as_raw_fd(), libc::STDIN_FILENO) };
        }
        let _ = write_termios(self.real.as_raw_fd(), &self.original);
    }
}

fn bridge() -> std::sync::MutexGuard<'static, Option<Bridge>> {
    BRIDGE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub(crate) fn installed() -> bool {
    bridge().is_some()
}

const KEYBOARD_QUERY_TIMEOUT: Duration = Duration::from_millis(2_000);
const KEYBOARD_FLAGS_MAX_DIGITS: usize = 3;
const BRACKETED_PASTE_START: &[u8] = b"\x1b[200~";
const BRACKETED_PASTE_END: &[u8] = b"\x1b[201~";

/// Query through a private nonblocking endpoint tied to the bridge's real tty.
///
/// A failed bridged query is not evidence that the terminal lacks support. The
/// caller conservatively declines to enable the protocol when this returns an
/// error. The bridge path checks its private two-second deadline through setup
/// and query I/O. The unbridged Session path keeps Crossterm's legacy reader and
/// deadline behavior; this bridge path never falls back to that detector.
pub(crate) fn probe_keyboard_enhancement() -> io::Result<bool> {
    let deadline = Instant::now()
        .checked_add(KEYBOARD_QUERY_TIMEOUT)
        .ok_or_else(|| {
            io::Error::new(ErrorKind::InvalidInput, "keyboard query deadline overflow")
        })?;
    {
        let mut active = bridge();
        let Some(active) = active.as_mut() else {
            return Err(io::Error::new(
                ErrorKind::Unsupported,
                "esc bridge is not installed",
            ));
        };
        let real_fd = active.real.as_raw_fd();
        probe_keyboard_enhancement_with(
            &mut active.stash,
            deadline,
            ProbeIo {
                now: Instant::now,
                open_endpoint: |_remaining: Duration| open_probe_endpoint(real_fd, deadline),
                endpoint_matches_real_tty: |endpoint: &mut OwnedFd, _remaining: Duration| {
                    endpoint_matches_real_tty(real_fd, endpoint.as_raw_fd())
                },
                output_is_ready: |endpoint: &mut OwnedFd, remaining: Duration| {
                    probe_endpoint_is_ready(endpoint.as_raw_fd(), libc::POLLOUT, remaining)
                },
                write_endpoint: |endpoint: &mut OwnedFd, bytes: &[u8]| {
                    probe_endpoint_write(endpoint.as_raw_fd(), bytes)
                },
                input_is_ready: |endpoint: &mut OwnedFd, remaining: Duration| {
                    probe_endpoint_is_ready(endpoint.as_raw_fd(), libc::POLLIN, remaining)
                },
                read_endpoint: |endpoint: &mut OwnedFd, buffer: &mut [u8]| {
                    probe_endpoint_read(endpoint.as_raw_fd(), buffer)
                },
            },
        )
    }
}

struct ProbeCapture<'stash> {
    stash: &'stash mut Vec<u8>,
    received: Vec<u8>,
    consume_on_success: Option<Range<usize>>,
}

impl<'stash> ProbeCapture<'stash> {
    fn new(stash: &'stash mut Vec<u8>) -> Self {
        Self {
            stash,
            received: Vec::new(),
            consume_on_success: None,
        }
    }
}

impl Drop for ProbeCapture<'_> {
    fn drop(&mut self) {
        if let Some(response) = self.consume_on_success.take()
            && response.start <= response.end
            && response.end <= self.received.len()
        {
            self.received.drain(response);
        }
        self.stash.append(&mut self.received);
    }
}

struct ProbeIo<Now, Open, Verify, OutputReady, Write, InputReady, Read> {
    now: Now,
    open_endpoint: Open,
    endpoint_matches_real_tty: Verify,
    output_is_ready: OutputReady,
    write_endpoint: Write,
    input_is_ready: InputReady,
    read_endpoint: Read,
}

fn probe_keyboard_enhancement_with<E, Now, Open, Verify, OutputReady, Write, InputReady, Read>(
    stash: &mut Vec<u8>,
    deadline: Instant,
    mut ops: ProbeIo<Now, Open, Verify, OutputReady, Write, InputReady, Read>,
) -> io::Result<bool>
where
    Now: FnMut() -> Instant,
    Open: FnMut(Duration) -> io::Result<E>,
    Verify: FnMut(&mut E, Duration) -> io::Result<bool>,
    OutputReady: FnMut(&mut E, Duration) -> io::Result<bool>,
    Write: FnMut(&mut E, &[u8]) -> io::Result<usize>,
    InputReady: FnMut(&mut E, Duration) -> io::Result<bool>,
    Read: FnMut(&mut E, &mut [u8]) -> io::Result<usize>,
{
    let mut endpoint = loop {
        let remaining = deadline.saturating_duration_since((ops.now)());
        if remaining.is_zero() {
            return Err(io::Error::new(
                ErrorKind::TimedOut,
                "keyboard query endpoint setup deadline elapsed",
            ));
        }
        match (ops.open_endpoint)(remaining) {
            Ok(endpoint) => break endpoint,
            Err(error) if error.kind() == ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        }
    };
    loop {
        let remaining = deadline.saturating_duration_since((ops.now)());
        if remaining.is_zero() {
            return Err(io::Error::new(
                ErrorKind::TimedOut,
                "keyboard query endpoint validation deadline elapsed",
            ));
        }
        match (ops.endpoint_matches_real_tty)(&mut endpoint, remaining) {
            Ok(true) => break,
            Ok(false) => {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "keyboard query endpoint is not the bridge tty",
                ));
            }
            Err(error) if error.kind() == ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        }
    }

    let mut capture = ProbeCapture::new(stash);
    let stale_len = capture.stash.len();
    let mut scanner = KeyboardReplyScanner::default();
    let _ = scanner.scan(capture.stash, 0, stale_len);

    let mut written = 0;
    while written < QUERY.len() {
        let remaining = deadline.saturating_duration_since((ops.now)());
        if remaining.is_zero() {
            return if written == 0 {
                Err(io::Error::new(
                    ErrorKind::TimedOut,
                    "keyboard query output deadline elapsed",
                ))
            } else {
                Ok(false)
            };
        }
        let ready = match (ops.output_is_ready)(&mut endpoint, remaining) {
            Ok(ready) => ready,
            Err(error)
                if error.kind() == ErrorKind::Interrupted
                    || error.kind() == ErrorKind::WouldBlock =>
            {
                continue;
            }
            Err(error) => return Err(error),
        };
        if (ops.now)() >= deadline {
            return if written == 0 {
                Err(io::Error::new(
                    ErrorKind::TimedOut,
                    "keyboard query output deadline elapsed",
                ))
            } else {
                Ok(false)
            };
        }
        if !ready {
            continue;
        }
        let count = match (ops.write_endpoint)(&mut endpoint, &QUERY.as_bytes()[written..]) {
            Ok(count) => count,
            Err(error)
                if error.kind() == ErrorKind::Interrupted
                    || error.kind() == ErrorKind::WouldBlock =>
            {
                continue;
            }
            Err(error) => return Err(error),
        };
        if count == 0 && written == 0 {
            return Err(io::Error::new(
                ErrorKind::WriteZero,
                "keyboard query write made no progress",
            ));
        }
        if count == 0 || count > QUERY.len() - written {
            return Err(io::Error::new(
                if count == 0 {
                    ErrorKind::WriteZero
                } else {
                    ErrorKind::InvalidData
                },
                "invalid keyboard query write count",
            ));
        }
        written += count;
        if (ops.now)() >= deadline {
            return Ok(false);
        }
    }

    let mut buffer = [0u8; 1024];
    loop {
        let remaining = deadline.saturating_duration_since((ops.now)());
        if remaining.is_zero() {
            return Ok(false);
        }
        let ready = match (ops.input_is_ready)(&mut endpoint, remaining) {
            Ok(ready) => ready,
            Err(error)
                if error.kind() == ErrorKind::Interrupted
                    || error.kind() == ErrorKind::WouldBlock =>
            {
                continue;
            }
            Err(error) => return Err(error),
        };
        if (ops.now)() >= deadline {
            return Ok(false);
        }
        if !ready {
            continue;
        }
        let count = match (ops.read_endpoint)(&mut endpoint, &mut buffer) {
            Ok(count) => count,
            Err(error)
                if error.kind() == ErrorKind::Interrupted
                    || error.kind() == ErrorKind::WouldBlock =>
            {
                continue;
            }
            Err(error) => return Err(error),
        };
        if count == 0 {
            return Err(io::Error::new(
                ErrorKind::UnexpectedEof,
                "real tty input closed",
            ));
        }
        if count > buffer.len() {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "real tty read exceeded its buffer",
            ));
        }
        let start = stale_len + capture.received.len();
        capture.received.extend_from_slice(&buffer[..count]);
        if (ops.now)() >= deadline {
            return Ok(false);
        }
        if let Some(response) = scanner.scan(&buffer[..count], start, stale_len) {
            if (ops.now)() >= deadline {
                return Ok(false);
            }
            capture.consume_on_success = Some(response.start - stale_len..response.end - stale_len);
            return Ok(true);
        }
    }
}

const TTY_NAME_BUFFER_CAPACITY: usize = 4096;

fn tty_path_for_fd(fd: RawFd) -> io::Result<Vec<u8>> {
    let mut path = vec![0; TTY_NAME_BUFFER_CAPACITY];
    // SAFETY: `fd` is the bridge-owned tty and `path` is writable C-string storage.
    let result = unsafe { libc::ttyname_r(fd, path.as_mut_ptr().cast(), path.len()) };
    if result != 0 {
        return Err(io::Error::from_raw_os_error(result));
    }
    let Some(end) = path.iter().position(|byte| *byte == 0) else {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "ttyname_r returned an unterminated path",
        ));
    };
    if end == 0 || path[0] != b'/' {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "ttyname_r did not return an absolute tty path",
        ));
    }
    path.truncate(end + 1);
    Ok(path)
}

fn open_probe_endpoint(real_fd: RawFd, deadline: Instant) -> io::Result<OwnedFd> {
    open_probe_endpoint_with(
        real_fd,
        deadline,
        Instant::now,
        tty_path_for_fd,
        open_probe_endpoint_path,
    )
}

fn open_probe_endpoint_with<E>(
    real_fd: RawFd,
    deadline: Instant,
    mut now: impl FnMut() -> Instant,
    mut resolve_path: impl FnMut(RawFd) -> io::Result<Vec<u8>>,
    mut open_path: impl FnMut(&[u8]) -> io::Result<E>,
) -> io::Result<E> {
    let path = resolve_path(real_fd)?;
    if now() >= deadline {
        return Err(io::Error::new(
            ErrorKind::TimedOut,
            "keyboard probe deadline elapsed while resolving the tty path",
        ));
    }
    open_path(&path)
}

fn open_probe_endpoint_path(path: &[u8]) -> io::Result<OwnedFd> {
    if path.len() < 2
        || path[0] != b'/'
        || path.last() != Some(&0)
        || path[..path.len() - 1].contains(&0)
    {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            "tty path must be an absolute NUL-terminated path",
        ));
    }
    let flags = libc::O_RDWR | libc::O_NONBLOCK | libc::O_CLOEXEC | libc::O_NOCTTY;
    // SAFETY: `path` was validated as an absolute NUL-terminated path. The
    // flags create an independent nonblocking descriptor without acquiring a
    // controlling tty.
    let fd = unsafe { libc::open(path.as_ptr().cast(), flags) };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: the successful `open` returned a new descriptor owned by this call.
    Ok(unsafe { OwnedFd::from_raw_fd(fd) })
}

fn endpoint_matches_real_tty(real_fd: RawFd, endpoint_fd: RawFd) -> io::Result<bool> {
    fn stat_fd(fd: RawFd) -> io::Result<libc::stat> {
        let mut stat = std::mem::MaybeUninit::<libc::stat>::uninit();
        // SAFETY: `stat` is valid writable storage for `fstat`.
        if unsafe { libc::fstat(fd, stat.as_mut_ptr()) } < 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: successful `fstat` initialized the structure.
        Ok(unsafe { stat.assume_init() })
    }

    let real = stat_fd(real_fd)?;
    let endpoint = stat_fd(endpoint_fd)?;
    let (Some(real), Some(endpoint)) = (character_device_id(&real), character_device_id(&endpoint))
    else {
        return Ok(false);
    };
    if !same_character_device(real, endpoint) {
        return Ok(false);
    }
    // F_GETFL observes only the private open description; no caller/bridge flags are changed.
    // SAFETY: `endpoint_fd` is the owned descriptor created with O_NONBLOCK.
    let flags = unsafe { libc::fcntl(endpoint_fd, libc::F_GETFL) };
    if flags < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(flags & libc::O_NONBLOCK != 0)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CharacterDeviceId {
    device: libc::dev_t,
    special_device: libc::dev_t,
}

fn character_device_id(stat: &libc::stat) -> Option<CharacterDeviceId> {
    (stat.st_mode & libc::S_IFMT == libc::S_IFCHR).then_some(CharacterDeviceId {
        device: stat.st_dev,
        special_device: stat.st_rdev,
    })
}

fn same_character_device(real: CharacterDeviceId, endpoint: CharacterDeviceId) -> bool {
    real == endpoint
}

fn probe_endpoint_is_ready(fd: RawFd, events: i16, timeout: Duration) -> io::Result<bool> {
    let mut pollfd = libc::pollfd {
        fd,
        events,
        revents: 0,
    };
    // Round down; the caller retries with the remaining absolute deadline.
    let millis = i32::try_from(timeout.as_millis()).unwrap_or(i32::MAX);
    // SAFETY: `pollfd` points to one initialized descriptor owned by the probe.
    let result = unsafe { libc::poll(&mut pollfd, 1, millis) };
    if result < 0 {
        return Err(io::Error::last_os_error());
    }
    if pollfd.revents & libc::POLLNVAL != 0 {
        return Err(io::Error::from_raw_os_error(libc::EBADF));
    }
    Ok(result > 0 && (pollfd.revents & (events | libc::POLLHUP | libc::POLLERR) != 0))
}

fn probe_endpoint_write(fd: RawFd, query: &[u8]) -> io::Result<usize> {
    // SAFETY: `query` is readable and `fd` is the owned O_NONBLOCK endpoint.
    let count = unsafe { libc::write(fd, query.as_ptr().cast(), query.len()) };
    if count < 0 {
        return Err(io::Error::last_os_error());
    }
    usize::try_from(count)
        .map_err(|_| io::Error::new(ErrorKind::InvalidData, "invalid keyboard query write count"))
}

fn probe_endpoint_read(fd: RawFd, buffer: &mut [u8]) -> io::Result<usize> {
    // SAFETY: `buffer` is writable and `fd` is the owned O_NONBLOCK endpoint.
    let count = unsafe { libc::read(fd, buffer.as_mut_ptr().cast(), buffer.len()) };
    if count < 0 {
        return Err(io::Error::last_os_error());
    }
    usize::try_from(count)
        .map_err(|_| io::Error::new(ErrorKind::InvalidData, "invalid tty read count"))
}

#[derive(Default)]
struct KeyboardReplyScanner {
    candidate: Vec<u8>,
    candidate_start: usize,
    in_bracketed_paste: bool,
}

impl KeyboardReplyScanner {
    fn scan(
        &mut self,
        bytes: &[u8],
        base_index: usize,
        fresh_boundary: usize,
    ) -> Option<Range<usize>> {
        for (offset, byte) in bytes.iter().copied().enumerate() {
            let index = base_index + offset;
            if self.candidate.is_empty() {
                if byte == 0x1b {
                    self.candidate.push(byte);
                    self.candidate_start = index;
                }
                continue;
            }
            if !self.accepts_next(byte) {
                self.candidate.clear();
                if byte == 0x1b {
                    self.candidate.push(byte);
                    self.candidate_start = index;
                }
                continue;
            }
            self.candidate.push(byte);
            if self.in_bracketed_paste {
                if self.candidate.as_slice() == BRACKETED_PASTE_END {
                    self.candidate.clear();
                    self.in_bracketed_paste = false;
                }
                continue;
            }
            if self.candidate.as_slice() == BRACKETED_PASTE_START {
                self.candidate.clear();
                self.in_bracketed_paste = true;
                continue;
            }
            if byte == b'u' {
                let response = parse_flags_reply(&self.candidate).is_some();
                let range = self.candidate_start..index + 1;
                self.candidate.clear();
                if response && range.start >= fresh_boundary {
                    return Some(range);
                }
            }
        }
        None
    }

    fn accepts_next(&self, byte: u8) -> bool {
        if self.in_bracketed_paste {
            return BRACKETED_PASTE_END.get(self.candidate.len()) == Some(&byte);
        }
        match self.candidate.len() {
            1 => byte == b'[',
            2 => byte == b'?' || byte == b'2',
            3 if self.candidate[2] == b'?' => byte.is_ascii_digit(),
            3 => byte == b'0',
            4..=6 if self.candidate[2] == b'?' => {
                let digits = self.candidate.len() - 3;
                byte == b'u' || (digits < KEYBOARD_FLAGS_MAX_DIGITS && byte.is_ascii_digit())
            }
            4 => byte == b'0',
            5 => byte == b'~',
            _ => false,
        }
    }
}

/// Point stdin at a side pty and keep the real tty for reading.
///
/// Call this before any `crossterm::event::poll` or `read`. Those bind the
/// process-wide reader to stdin. Swapping stdin afterwards leaves that reader
/// on the wrong descriptor, so application keys never arrive.
pub(crate) fn install() -> io::Result<()> {
    install_with(
        dup_stdin,
        read_termios,
        make_raw,
        open_raw_pty,
        set_cloexec,
        replace_stdin,
    )
}

fn install_with(
    dup_stdin_fn: impl FnOnce() -> io::Result<OwnedFd>,
    read_termios_fn: impl FnOnce(RawFd) -> io::Result<libc::termios>,
    make_raw_fn: impl FnOnce(RawFd) -> io::Result<()>,
    open_pty: impl FnOnce() -> io::Result<(OwnedFd, OwnedFd)>,
    mut set_cloexec_fn: impl FnMut(RawFd) -> io::Result<()>,
    replace_stdin_fn: impl FnOnce(RawFd) -> io::Result<()>,
) -> io::Result<()> {
    if bridge().is_some() {
        return Ok(());
    }
    let real = dup_stdin_fn()?;
    let original = read_termios_fn(real.as_raw_fd())?;
    let mut rollback = RestoreOnFailure {
        real: &real,
        original,
        stdin_touched: false,
        armed: true,
    };
    make_raw_fn(real.as_raw_fd())?;
    let (master, slave) = open_pty()?;
    set_cloexec_fn(real.as_raw_fd())?;
    set_cloexec_fn(master.as_raw_fd())?;

    // All setup that can fail is complete. Mark stdin before the final syscall
    // so the guard also restores it if the replacement reports an error.
    rollback.stdin_touched = true;
    replace_stdin_fn(slave.as_raw_fd())?;

    rollback.armed = false;
    drop(rollback);
    *bridge() = Some(Bridge {
        real,
        master,
        stash: Vec::new(),
        original,
    });
    Ok(())
}

/// Discard bytes already queued on the real tty. Does not touch Crossterm.
pub(crate) fn drain_real() -> io::Result<usize> {
    let mut count = 0;
    let mut buf = [0u8; 1024];
    while real_is_readable(Duration::ZERO)? {
        let n = read_real(&mut buf)?;
        if n == 0 {
            break;
        }
        count += n;
    }
    Ok(count)
}

/// Put the real tty back on stdin and restore the termios from `install`.
///
/// Run this after Crossterm's raw-mode restore. That restore writes the side
/// slave's saved mode onto whichever fd is stdin at the time.
pub(crate) fn restore_stdin() {
    let Some(bridge) = bridge().take() else {
        return;
    };
    // SAFETY: `bridge.real` is the dup of the tty stdin from `install`.
    let rc = unsafe { libc::dup2(bridge.real.as_raw_fd(), libc::STDIN_FILENO) };
    if rc >= 0 {
        let _ = write_termios(libc::STDIN_FILENO, &bridge.original);
    }
}

/// Wait until Crossterm has one event, or until `timeout` elapses.
///
/// `None` waits until an event arrives. Bytes are moved from the real tty to
/// the side pty, splitting the first `\x1b\x1b` pair so the second ESC stays
/// the start of the next sequence.
pub(crate) fn wait_for_event(timeout: Option<Duration>) -> io::Result<bool> {
    let start = Instant::now();
    loop {
        if crossterm_poll(Duration::ZERO)? {
            return Ok(true);
        }
        pump_stash()?;
        if crossterm_poll(Duration::ZERO)? {
            return Ok(true);
        }
        let slice = match timeout {
            None => Duration::from_millis(50),
            Some(limit) => {
                let left = limit.saturating_sub(start.elapsed());
                if left.is_zero() {
                    flush_held_esc()?;
                    return crossterm_poll(Duration::ZERO);
                }
                left.min(Duration::from_millis(50))
            }
        };
        if real_is_readable(slice)? {
            let mut buf = [0u8; 1024];
            let count = read_real(&mut buf)?;
            if count == 0 {
                return Err(io::Error::new(ErrorKind::UnexpectedEof, "tty input closed"));
            }
            queue_bytes(&buf[..count])?;
            continue;
        }
        // No more bytes followed the held second ESC. Deliver it as Esc.
        flush_held_esc()?;
        if crossterm_poll(Duration::ZERO)? {
            return Ok(true);
        }
        if timeout.is_some_and(|limit| start.elapsed() >= limit) {
            return Ok(false);
        }
    }
}

fn pump_stash() -> io::Result<()> {
    let Some(bytes) = take_ready_stash() else {
        return Ok(());
    };
    write_master(&bytes)
}

/// A stash that is only a second ESC stays held until another byte arrives,
/// so Alt+x is not split into Esc and `x`.
fn take_ready_stash() -> Option<Vec<u8>> {
    let mut guard = bridge();
    let bridge = guard.as_mut()?;
    if bridge.stash.is_empty() || bridge.stash == [0x1b] {
        return None;
    }
    let bytes = std::mem::take(&mut bridge.stash);
    if let Some(second) = second_esc_index(&bytes) {
        bridge.stash = bytes[second..].to_vec();
        Some(bytes[..second].to_vec())
    } else {
        Some(bytes)
    }
}

fn queue_bytes(bytes: &[u8]) -> io::Result<()> {
    let pending = {
        let mut guard = bridge();
        let Some(bridge) = guard.as_mut() else {
            return Ok(());
        };
        if bridge.stash == [0x1b] {
            let mut combined = std::mem::take(&mut bridge.stash);
            combined.extend_from_slice(bytes);
            combined
        } else if bridge.stash.is_empty() {
            bytes.to_vec()
        } else {
            bridge.stash.extend_from_slice(bytes);
            std::mem::take(&mut bridge.stash)
        }
    };
    if let Some(second) = second_esc_index(&pending) {
        write_master(&pending[..second])?;
        if let Some(bridge) = bridge().as_mut() {
            bridge.stash.extend_from_slice(&pending[second..]);
        }
    } else {
        write_master(&pending)?;
    }
    Ok(())
}

fn flush_held_esc() -> io::Result<()> {
    let held = {
        let mut guard = bridge();
        let Some(bridge) = guard.as_mut() else {
            return Ok(());
        };
        if bridge.stash == [0x1b] {
            bridge.stash.clear();
            true
        } else {
            false
        }
    };
    if held {
        write_master(&[0x1b])?;
    }
    Ok(())
}

fn real_is_readable(timeout: Duration) -> io::Result<bool> {
    let fd = bridge()
        .as_ref()
        .map(|bridge| bridge.real.as_raw_fd())
        .ok_or_else(|| io::Error::other("esc bridge is not installed"))?;
    let mut pollfd = libc::pollfd {
        fd,
        events: libc::POLLIN,
        revents: 0,
    };
    let millis = i32::try_from(timeout.as_millis()).unwrap_or(i32::MAX);
    loop {
        // SAFETY: `pollfd` points at one struct for this fd, which `install` opened.
        let rc = unsafe { libc::poll(&mut pollfd, 1, millis) };
        if rc < 0 {
            let err = io::Error::last_os_error();
            if err.kind() == ErrorKind::Interrupted {
                continue;
            }
            return Err(err);
        }
        return Ok(rc > 0 && pollfd.revents & libc::POLLIN != 0);
    }
}

fn read_real(buf: &mut [u8]) -> io::Result<usize> {
    let fd = bridge()
        .as_ref()
        .map(|bridge| bridge.real.as_raw_fd())
        .ok_or_else(|| io::Error::other("esc bridge is not installed"))?;
    loop {
        // SAFETY: `buf` is a writable buffer and `fd` is the real tty.
        let count = unsafe { libc::read(fd, buf.as_mut_ptr().cast(), buf.len()) };
        if count < 0 {
            let err = io::Error::last_os_error();
            if err.kind() == ErrorKind::Interrupted {
                continue;
            }
            return Err(err);
        }
        return Ok(usize::try_from(count).unwrap_or(0));
    }
}

fn write_master(bytes: &[u8]) -> io::Result<()> {
    if bytes.is_empty() {
        return Ok(());
    }
    let fd = bridge()
        .as_ref()
        .map(|bridge| bridge.master.as_raw_fd())
        .ok_or_else(|| io::Error::other("esc bridge is not installed"))?;
    let mut offset = 0;
    while offset < bytes.len() {
        // SAFETY: `bytes` is the caller's chunk and `fd` is the side pty master.
        let count =
            unsafe { libc::write(fd, bytes[offset..].as_ptr().cast(), bytes.len() - offset) };
        if count < 0 {
            let err = io::Error::last_os_error();
            if err.kind() == ErrorKind::Interrupted {
                continue;
            }
            return Err(err);
        }
        offset += usize::try_from(count).unwrap_or(0);
    }
    Ok(())
}

fn dup_stdin() -> io::Result<OwnedFd> {
    // SAFETY: duplicating stdin. Ownership of the new descriptor is transferred below.
    let fd = unsafe { libc::dup(libc::STDIN_FILENO) };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: `dup` returned a new valid descriptor owned by this function.
    let fd = unsafe { OwnedFd::from_raw_fd(fd) };
    // SAFETY: `fd` is an open descriptor returned by `dup`.
    if unsafe { libc::isatty(fd.as_raw_fd()) } != 1 {
        return Err(io::Error::new(ErrorKind::Unsupported, "stdin is not a tty"));
    }
    Ok(fd)
}

fn open_raw_pty() -> io::Result<(OwnedFd, OwnedFd)> {
    let (master, slave) = open_pty()?;
    make_raw(slave.as_raw_fd())?;
    Ok((master, slave))
}

fn open_pty() -> io::Result<(OwnedFd, OwnedFd)> {
    let mut master = 0;
    let mut slave = 0;
    // SAFETY: openpty writes the two fds. Null name, termios, and size keep
    // the implementation defaults.
    let rc = unsafe {
        libc::openpty(
            &mut master,
            &mut slave,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    if rc != 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: successful `openpty` returned two new open descriptors owned here.
    let master = unsafe { OwnedFd::from_raw_fd(master) };
    // SAFETY: successful `openpty` returned two new open descriptors owned here.
    let slave = unsafe { OwnedFd::from_raw_fd(slave) };
    Ok((master, slave))
}

fn read_termios(fd: RawFd) -> io::Result<libc::termios> {
    // SAFETY: `tcgetattr` fills `attrs` for this open tty fd.
    unsafe {
        let mut attrs = std::mem::MaybeUninit::<libc::termios>::uninit();
        if libc::tcgetattr(fd, attrs.as_mut_ptr()) != 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(attrs.assume_init())
    }
}

fn write_termios(fd: RawFd, attrs: &libc::termios) -> io::Result<()> {
    // SAFETY: `attrs` is a complete termios value and `fd` is an open tty.
    if unsafe { libc::tcsetattr(fd, libc::TCSANOW, attrs) } != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

fn make_raw(fd: RawFd) -> io::Result<()> {
    // SAFETY: `tcgetattr` fills `attrs` for this open tty fd.
    let mut attrs = unsafe {
        let mut attrs = std::mem::MaybeUninit::<libc::termios>::uninit();
        if libc::tcgetattr(fd, attrs.as_mut_ptr()) != 0 {
            return Err(io::Error::last_os_error());
        }
        attrs.assume_init()
    };
    // SAFETY: `cfmakeraw` only rewrites the termios flags in `attrs`.
    unsafe { libc::cfmakeraw(&mut attrs) };
    // SAFETY: `attrs` came from `tcgetattr` on this same fd.
    if unsafe { libc::tcsetattr(fd, libc::TCSANOW, &attrs) } != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

fn set_cloexec(fd: RawFd) -> io::Result<()> {
    // SAFETY: `fd` is one of the bridge fds just opened.
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
    if flags < 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: same fd, setting the close-on-exec bit only.
    if unsafe { libc::fcntl(fd, libc::F_SETFD, flags | libc::FD_CLOEXEC) } < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

fn replace_stdin(fd: RawFd) -> io::Result<()> {
    // SAFETY: `fd` is the open side-pty slave descriptor owned by `install`.
    if unsafe { libc::dup2(fd, libc::STDIN_FILENO) } < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};
    use std::collections::VecDeque;
    use std::io;
    use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
    use std::panic::{AssertUnwindSafe, catch_unwind};
    use std::rc::Rc;
    use std::time::{Duration, Instant};

    use super::{
        CharacterDeviceId, ProbeIo, QUERY, dup_stdin, endpoint_matches_real_tty, install,
        install_with, installed, make_raw, open_probe_endpoint, open_probe_endpoint_path,
        open_probe_endpoint_with, open_pty, open_raw_pty, probe_keyboard_enhancement_with,
        read_termios, replace_stdin, restore_stdin, same_character_device, second_esc_index,
        set_cloexec, tty_path_for_fd, write_termios,
    };

    struct StdinFixture {
        original_stdin: OwnedFd,
        _master: OwnedFd,
        slave: OwnedFd,
        original_termios: libc::termios,
    }

    impl StdinFixture {
        fn new() -> io::Result<Self> {
            let original_stdin = duplicate_fd(libc::STDIN_FILENO)?;
            let (master, slave) = open_pty()?;
            replace_stdin(slave.as_raw_fd())?;
            let requested_termios = match read_termios(slave.as_raw_fd()) {
                Ok(attrs) => attrs,
                Err(error) => {
                    let _ = replace_stdin(original_stdin.as_raw_fd());
                    return Err(error);
                }
            };
            // macOS reports PENDIN after canonical mode is restored. Capture
            // the kernel-effective state after one mode cycle instead of
            // masking that stateful bit in the assertions below.
            let original_termios = match (|| {
                make_raw(slave.as_raw_fd())?;
                write_termios(slave.as_raw_fd(), &requested_termios)?;
                read_termios(slave.as_raw_fd())
            })() {
                Ok(attrs) => attrs,
                Err(error) => {
                    let _ = write_termios(slave.as_raw_fd(), &requested_termios);
                    let _ = replace_stdin(original_stdin.as_raw_fd());
                    return Err(error);
                }
            };
            Ok(Self {
                original_stdin,
                _master: master,
                slave,
                original_termios,
            })
        }

        fn assert_stdin_is_fixture_slave(&self) -> io::Result<()> {
            assert!(same_open_file(libc::STDIN_FILENO, self.slave.as_raw_fd())?);
            Ok(())
        }

        fn assert_termios_restored(&self, context: impl std::fmt::Debug) -> io::Result<()> {
            let actual = read_termios(self.slave.as_raw_fd())?;
            assert_termios_equal(&actual, &self.original_termios, context);
            Ok(())
        }
    }

    impl Drop for StdinFixture {
        fn drop(&mut self) {
            if installed() {
                restore_stdin();
            }
            let _ = replace_stdin(self.original_stdin.as_raw_fd());
            let _ = write_termios(self.slave.as_raw_fd(), &self.original_termios);
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum FailureAt {
        DuplicateStdin,
        ReadTermios,
        MakeRaw,
        OpenPty,
        CloseOnExec(usize),
        ReplaceStdin,
    }

    fn duplicate_fd(fd: RawFd) -> io::Result<OwnedFd> {
        // SAFETY: `fd` is open and `dup` returns a new descriptor on success.
        let duplicate = unsafe { libc::dup(fd) };
        if duplicate < 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: `dup` returned a new descriptor owned by this function.
        Ok(unsafe { OwnedFd::from_raw_fd(duplicate) })
    }

    fn same_open_file(left_fd: RawFd, right_fd: RawFd) -> io::Result<bool> {
        // SAFETY: `fstat` initializes each value for the supplied open descriptor.
        let left = unsafe {
            let mut stat = std::mem::MaybeUninit::<libc::stat>::uninit();
            if libc::fstat(left_fd, stat.as_mut_ptr()) != 0 {
                return Err(io::Error::last_os_error());
            }
            stat.assume_init()
        };
        // SAFETY: `fstat` initializes each value for the supplied open descriptor.
        let right = unsafe {
            let mut stat = std::mem::MaybeUninit::<libc::stat>::uninit();
            if libc::fstat(right_fd, stat.as_mut_ptr()) != 0 {
                return Err(io::Error::last_os_error());
            }
            stat.assume_init()
        };
        Ok(left.st_dev == right.st_dev
            && left.st_ino == right.st_ino
            && left.st_rdev == right.st_rdev)
    }

    fn assert_termios_equal(
        actual: &libc::termios,
        expected: &libc::termios,
        context: impl std::fmt::Debug,
    ) {
        assert_eq!(actual.c_iflag, expected.c_iflag, "stage: {context:?}");
        assert_eq!(actual.c_oflag, expected.c_oflag, "stage: {context:?}");
        assert_eq!(actual.c_cflag, expected.c_cflag, "stage: {context:?}");
        assert_eq!(actual.c_lflag, expected.c_lflag, "stage: {context:?}");
        assert_eq!(actual.c_cc, expected.c_cc, "stage: {context:?}");
        assert_eq!(actual.c_ispeed, expected.c_ispeed, "stage: {context:?}");
        assert_eq!(actual.c_ospeed, expected.c_ospeed, "stage: {context:?}");
    }

    fn assert_closed(fd: RawFd) {
        // SAFETY: F_GETFD only inspects this descriptor number.
        let result = unsafe { libc::fcntl(fd, libc::F_GETFD) };
        assert_eq!(result, -1, "descriptor {fd} should have been closed");
        assert_eq!(
            io::Error::last_os_error().raw_os_error(),
            Some(libc::EBADF),
            "descriptor {fd} should fail with EBADF"
        );
    }

    fn injected_error(errno: i32) -> io::Error {
        io::Error::from_raw_os_error(errno)
    }

    fn assert_install_failure_rolls_back(failure_at: FailureAt, errno: i32) -> io::Result<()> {
        let fixture = StdinFixture::new()?;
        let opened_fds = Rc::new(RefCell::new(Vec::new()));
        let cloexec_calls = Rc::new(Cell::new(0));

        let duplicate_fds = Rc::clone(&opened_fds);
        let read_fds = Rc::clone(&opened_fds);
        let raw_fds = Rc::clone(&opened_fds);
        let pty_fds = Rc::clone(&opened_fds);
        let cloexec_fds = Rc::clone(&opened_fds);
        let replace_fds = Rc::clone(&opened_fds);
        let calls = Rc::clone(&cloexec_calls);

        let result = install_with(
            move || {
                if failure_at == FailureAt::DuplicateStdin {
                    return Err(injected_error(errno));
                }
                let fd = dup_stdin()?;
                duplicate_fds.borrow_mut().push(fd.as_raw_fd());
                Ok(fd)
            },
            move |fd| {
                read_fds.borrow_mut().push(fd);
                if failure_at == FailureAt::ReadTermios {
                    Err(injected_error(errno))
                } else {
                    read_termios(fd)
                }
            },
            move |fd| {
                raw_fds.borrow_mut().push(fd);
                make_raw(fd)?;
                if failure_at == FailureAt::MakeRaw {
                    Err(injected_error(errno))
                } else {
                    Ok(())
                }
            },
            move || {
                let pair = open_raw_pty()?;
                pty_fds
                    .borrow_mut()
                    .extend([pair.0.as_raw_fd(), pair.1.as_raw_fd()]);
                if failure_at == FailureAt::OpenPty {
                    drop(pair);
                    Err(injected_error(errno))
                } else {
                    Ok(pair)
                }
            },
            move |fd| {
                cloexec_fds.borrow_mut().push(fd);
                let call = calls.get() + 1;
                calls.set(call);
                if failure_at == FailureAt::CloseOnExec(call) {
                    Err(injected_error(errno))
                } else {
                    set_cloexec(fd)
                }
            },
            move |fd| {
                replace_fds.borrow_mut().push(fd);
                replace_stdin(fd)?;
                if failure_at == FailureAt::ReplaceStdin {
                    Err(injected_error(errno))
                } else {
                    Ok(())
                }
            },
        );

        let error = match result {
            Ok(()) => panic!("install unexpectedly succeeded at injected failure {failure_at:?}"),
            Err(error) => error,
        };
        assert_eq!(error.raw_os_error(), Some(errno));
        assert!(!installed());
        fixture.assert_stdin_is_fixture_slave()?;
        fixture.assert_termios_restored(failure_at)?;
        for fd in opened_fds.borrow().iter().copied() {
            assert_closed(fd);
        }
        Ok(())
    }

    #[test]
    fn esc_esc_x_keeps_the_second_escape_as_the_next_sequence() {
        assert_eq!(second_esc_index(b"\x1b\x1bx"), Some(1));
    }

    #[test]
    fn a_single_alt_chord_is_not_split() {
        assert_eq!(second_esc_index(b"\x1bx"), None);
    }

    #[test]
    fn the_split_is_the_first_pair_after_a_csi_sequence() {
        assert_eq!(second_esc_index(b"\x1b[A\x1b\x1bx"), Some(4));
    }

    #[test]
    fn failed_install_restores_stdin_termios_and_all_owned_descriptors() -> io::Result<()> {
        for (failure_at, errno) in [
            (FailureAt::DuplicateStdin, libc::EACCES),
            (FailureAt::ReadTermios, libc::EBUSY),
            (FailureAt::MakeRaw, libc::EIO),
            (FailureAt::OpenPty, libc::EINVAL),
            (FailureAt::CloseOnExec(1), libc::ENOTTY),
            (FailureAt::CloseOnExec(2), libc::EAGAIN),
            (FailureAt::ReplaceStdin, libc::EPERM),
        ] {
            assert_install_failure_rolls_back(failure_at, errno)?;
        }

        let fixture = StdinFixture::new()?;
        install()?;
        assert!(installed());
        assert!(!same_open_file(
            libc::STDIN_FILENO,
            fixture.slave.as_raw_fd()
        )?);
        restore_stdin();
        assert!(!installed());
        fixture.assert_stdin_is_fixture_slave()?;
        fixture.assert_termios_restored("successful install and restore")?;
        Ok(())
    }

    #[derive(Clone, Debug, Default)]
    struct ProbeScript {
        chunks: VecDeque<Vec<u8>>,
        query: Vec<u8>,
        input_polls: usize,
        reads: usize,
    }

    struct ProbeEndpoint {
        drops: Rc<Cell<usize>>,
        nonblocking: bool,
    }

    impl Drop for ProbeEndpoint {
        fn drop(&mut self) {
            self.drops.set(self.drops.get() + 1);
        }
    }

    fn run_probe_with_chunks(
        chunks: Vec<Vec<u8>>,
        stash: &mut Vec<u8>,
    ) -> (io::Result<bool>, ProbeScript, usize, i32) {
        let script = Rc::new(RefCell::new(ProbeScript {
            chunks: chunks.into(),
            ..ProbeScript::default()
        }));
        let drops = Rc::new(Cell::new(0));
        let caller_flags = Rc::new(Cell::new(0x1234));
        let clock = Rc::new(Cell::new(Duration::ZERO));
        let now_clock = Rc::clone(&clock);
        let start = Instant::now();
        let deadline = start + Duration::from_secs(2);

        let open_drops = Rc::clone(&drops);
        let result = probe_keyboard_enhancement_with(
            stash,
            deadline,
            ProbeIo {
                now: move || start + now_clock.get(),
                open_endpoint: move |_: Duration| {
                    Ok(ProbeEndpoint {
                        drops: Rc::clone(&open_drops),
                        nonblocking: true,
                    })
                },
                endpoint_matches_real_tty: {
                    let caller_flags = Rc::clone(&caller_flags);
                    move |endpoint: &mut ProbeEndpoint, _: Duration| {
                        assert!(endpoint.nonblocking);
                        assert_eq!(caller_flags.get(), 0x1234);
                        Ok(true)
                    }
                },
                output_is_ready: |_: &mut ProbeEndpoint, _: Duration| Ok(true),
                write_endpoint: {
                    let script = Rc::clone(&script);
                    move |_: &mut ProbeEndpoint, bytes: &[u8]| {
                        script.borrow_mut().query.extend_from_slice(bytes);
                        Ok(bytes.len())
                    }
                },
                input_is_ready: {
                    let script = Rc::clone(&script);
                    let clock = Rc::clone(&clock);
                    move |_: &mut ProbeEndpoint, _: Duration| {
                        let mut script = script.borrow_mut();
                        script.input_polls += 1;
                        let ready = !script.chunks.is_empty();
                        if !ready {
                            clock.set(Duration::from_secs(2));
                        }
                        Ok(ready)
                    }
                },
                read_endpoint: {
                    let script = Rc::clone(&script);
                    move |_: &mut ProbeEndpoint, buffer: &mut [u8]| {
                        let mut script = script.borrow_mut();
                        script.reads += 1;
                        let Some(bytes) = script.chunks.pop_front() else {
                            return Ok(0);
                        };
                        buffer[..bytes.len()].copy_from_slice(&bytes);
                        Ok(bytes.len())
                    }
                },
            },
        );

        let snapshot = script.borrow().clone();
        (result, snapshot, drops.get(), caller_flags.get())
    }

    fn detected(result: io::Result<bool>) -> bool {
        result.expect("keyboard probe completes")
    }

    #[derive(Clone, Copy)]
    enum ProbeTermination {
        Success,
        Error,
        Panic,
    }

    #[test]
    fn physical_tty_path_reopens_the_bridge_device_without_changing_its_flags() -> io::Result<()> {
        let (_master, real) = open_pty()?;
        let real_fd = real.as_raw_fd();
        // SAFETY: F_GETFL only reads the open PTY descriptor's status flags.
        let original_flags = unsafe { libc::fcntl(real_fd, libc::F_GETFL) };
        assert!(original_flags >= 0);

        let path = tty_path_for_fd(real_fd)?;
        assert!(path.starts_with(b"/"));
        assert_eq!(path.last(), Some(&0_u8));
        let endpoint = open_probe_endpoint(real_fd, Instant::now() + Duration::from_secs(2))?;
        assert!(endpoint_matches_real_tty(real_fd, endpoint.as_raw_fd())?);
        // SAFETY: F_GETFL only observes this independently opened endpoint.
        let endpoint_flags = unsafe { libc::fcntl(endpoint.as_raw_fd(), libc::F_GETFL) };
        assert!(endpoint_flags >= 0);
        assert_ne!(endpoint_flags & libc::O_NONBLOCK, 0);
        // SAFETY: F_GETFL only reads the bridge-owned PTY descriptor.
        assert_eq!(
            unsafe { libc::fcntl(real_fd, libc::F_GETFL) },
            original_flags
        );

        let endpoint_fd = endpoint.as_raw_fd();
        drop(endpoint);
        assert_closed(endpoint_fd);
        Ok(())
    }

    #[test]
    fn indirect_tty_alias_numbers_do_not_prove_physical_device_identity() {
        let physical = CharacterDeviceId {
            device: 7,
            special_device: 42,
        };
        let indirect_alias = CharacterDeviceId {
            device: 7,
            special_device: 0,
        };
        assert!(!same_character_device(physical, indirect_alias));
        assert!(same_character_device(physical, physical));
    }

    #[test]
    fn tty_path_resolution_rechecks_the_absolute_deadline_before_open() -> io::Result<()> {
        let (_master, real) = open_pty()?;
        let real_fd = real.as_raw_fd();
        let base = Instant::now();
        let deadline = base + Duration::from_millis(100);
        let elapsed = Cell::new(Duration::ZERO);
        let opened = Cell::new(false);

        let result = open_probe_endpoint_with(
            real_fd,
            deadline,
            || base + elapsed.get(),
            |resolved_fd| {
                assert_eq!(resolved_fd, real_fd);
                elapsed.set(Duration::from_millis(100));
                Ok(b"/dev/never-open-this\0".to_vec())
            },
            |_: &[u8]| -> io::Result<OwnedFd> {
                opened.set(true);
                panic!("open must not run after path resolution spends the deadline")
            },
        );

        let error = result.expect_err("path resolution elapsed the deadline");
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
        assert!(!opened.get());
        Ok(())
    }

    #[test]
    fn physical_endpoint_resolution_and_identity_errors_are_preserved() -> io::Result<()> {
        let deadline = Instant::now() + Duration::from_secs(2);
        let resolution_error = open_probe_endpoint_with(
            -1,
            deadline,
            Instant::now,
            tty_path_for_fd,
            |_: &[u8]| -> io::Result<OwnedFd> {
                panic!("failed ttyname_r must not proceed to open")
            },
        )
        .expect_err("invalid bridge fd cannot resolve a physical tty name");
        assert_eq!(resolution_error.raw_os_error(), Some(libc::EBADF));

        let mut resolver_stash = b"resolver input stays untouched".to_vec();
        let result = probe_keyboard_enhancement_with(
            &mut resolver_stash,
            deadline,
            ProbeIo {
                now: Instant::now,
                open_endpoint: |_: Duration| open_probe_endpoint(-1, deadline),
                endpoint_matches_real_tty: |_: &mut OwnedFd, _: Duration| -> io::Result<bool> {
                    panic!("failed ttyname_r must not validate an endpoint")
                },
                output_is_ready: |_: &mut OwnedFd, _: Duration| -> io::Result<bool> {
                    panic!("failed ttyname_r must not poll output")
                },
                write_endpoint: |_: &mut OwnedFd, _: &[u8]| -> io::Result<usize> {
                    panic!("failed ttyname_r must not write a query")
                },
                input_is_ready: |_: &mut OwnedFd, _: Duration| -> io::Result<bool> {
                    panic!("failed ttyname_r must not poll input")
                },
                read_endpoint: |_: &mut OwnedFd, _: &mut [u8]| -> io::Result<usize> {
                    panic!("failed ttyname_r must not read")
                },
            },
        );
        assert_eq!(
            result
                .expect_err("tty-name resolution error reaches the probe caller")
                .raw_os_error(),
            Some(libc::EBADF)
        );
        assert_eq!(resolver_stash, b"resolver input stays untouched");

        let (_master, real) = open_pty()?;
        let (_different_master, different) = open_pty()?;
        let real_fd = real.as_raw_fd();
        let different_fd = different.as_raw_fd();
        let opened_endpoint = Cell::new(-1);
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut stash = b"preserved".to_vec();
        let result = probe_keyboard_enhancement_with(
            &mut stash,
            deadline,
            ProbeIo {
                now: Instant::now,
                open_endpoint: |_: Duration| {
                    let endpoint = open_probe_endpoint(real_fd, deadline)?;
                    opened_endpoint.set(endpoint.as_raw_fd());
                    Ok(endpoint)
                },
                endpoint_matches_real_tty: |endpoint: &mut OwnedFd, _: Duration| {
                    endpoint_matches_real_tty(different_fd, endpoint.as_raw_fd())
                },
                output_is_ready: |_: &mut OwnedFd, _: Duration| -> io::Result<bool> {
                    panic!("identity error must prevent polling")
                },
                write_endpoint: |_: &mut OwnedFd, _: &[u8]| -> io::Result<usize> {
                    panic!("identity error must prevent query output")
                },
                input_is_ready: |_: &mut OwnedFd, _: Duration| -> io::Result<bool> {
                    panic!("identity error must prevent input polling")
                },
                read_endpoint: |_: &mut OwnedFd, _: &mut [u8]| -> io::Result<usize> {
                    panic!("identity error must prevent reading")
                },
            },
        );

        assert_eq!(
            result
                .expect_err("a mismatched physical tty is a probe error")
                .kind(),
            io::ErrorKind::InvalidData
        );
        assert_eq!(stash, b"preserved");
        assert_closed(opened_endpoint.get());
        Ok(())
    }

    #[test]
    fn physical_endpoint_setup_retries_resolver_and_open_interrupted() -> io::Result<()> {
        let (_master, real) = open_pty()?;
        let real_fd = real.as_raw_fd();
        let resolver_calls = Cell::new(0);
        let open_calls = Cell::new(0);
        let query_written = Cell::new(false);
        let mut reply = Some(b"\x1b[?0u".to_vec());
        let mut stash = Vec::new();
        let deadline = Instant::now() + Duration::from_secs(2);

        let result = probe_keyboard_enhancement_with(
            &mut stash,
            deadline,
            ProbeIo {
                now: Instant::now,
                open_endpoint: |_: Duration| {
                    open_probe_endpoint_with(
                        real_fd,
                        deadline,
                        Instant::now,
                        |fd| {
                            let call = resolver_calls.get();
                            resolver_calls.set(call + 1);
                            if call == 0 {
                                Err(io::Error::from(io::ErrorKind::Interrupted))
                            } else {
                                tty_path_for_fd(fd)
                            }
                        },
                        |path| {
                            let call = open_calls.get();
                            open_calls.set(call + 1);
                            if call == 0 {
                                Err(io::Error::from(io::ErrorKind::Interrupted))
                            } else {
                                open_probe_endpoint_path(path)
                            }
                        },
                    )
                },
                endpoint_matches_real_tty: |endpoint: &mut OwnedFd, _: Duration| {
                    endpoint_matches_real_tty(real_fd, endpoint.as_raw_fd())
                },
                output_is_ready: |_: &mut OwnedFd, _: Duration| Ok(true),
                write_endpoint: |_: &mut OwnedFd, query: &[u8]| {
                    query_written.set(true);
                    Ok(query.len())
                },
                input_is_ready: |_: &mut OwnedFd, _: Duration| Ok(true),
                read_endpoint: |_: &mut OwnedFd, buffer: &mut [u8]| {
                    let bytes = reply.take().expect("one scripted flags reply");
                    buffer[..bytes.len()].copy_from_slice(&bytes);
                    Ok(bytes.len())
                },
            },
        );

        assert!(result?);
        assert_eq!(resolver_calls.get(), 3);
        assert_eq!(open_calls.get(), 2);
        assert!(query_written.get());
        assert!(stash.is_empty());
        Ok(())
    }

    #[test]
    fn keyboard_probe_preserves_prequery_output_errors() {
        #[derive(Clone, Copy)]
        enum Failure {
            Readiness,
            Write,
            ZeroWrite,
        }

        for (failure, expected_kind, expected_write_calls) in [
            (Failure::Readiness, io::ErrorKind::BrokenPipe, 0),
            (Failure::Write, io::ErrorKind::PermissionDenied, 1),
            (Failure::ZeroWrite, io::ErrorKind::WriteZero, 1),
        ] {
            let drops = Rc::new(Cell::new(0));
            let open_drops = Rc::clone(&drops);
            let write_calls = Cell::new(0);
            let mut stash = b"preexisting input".to_vec();
            let deadline = Instant::now() + Duration::from_secs(2);

            let result = probe_keyboard_enhancement_with(
                &mut stash,
                deadline,
                ProbeIo {
                    now: Instant::now,
                    open_endpoint: move |_: Duration| -> io::Result<ProbeEndpoint> {
                        Ok(ProbeEndpoint {
                            drops: Rc::clone(&open_drops),
                            nonblocking: true,
                        })
                    },
                    endpoint_matches_real_tty: |endpoint: &mut ProbeEndpoint,
                                                _: Duration|
                     -> io::Result<bool> {
                        assert!(endpoint.nonblocking);
                        Ok(true)
                    },
                    output_is_ready: move |_: &mut ProbeEndpoint,
                                           _: Duration|
                          -> io::Result<bool> {
                        if matches!(failure, Failure::Readiness) {
                            Err(io::Error::from(io::ErrorKind::BrokenPipe))
                        } else {
                            Ok(true)
                        }
                    },
                    write_endpoint: |_: &mut ProbeEndpoint, _bytes: &[u8]| -> io::Result<usize> {
                        write_calls.set(write_calls.get() + 1);
                        match failure {
                            Failure::Readiness => unreachable!("readiness failed first"),
                            Failure::Write => Err(io::Error::from(io::ErrorKind::PermissionDenied)),
                            Failure::ZeroWrite => Ok(0),
                        }
                    },
                    input_is_ready: |_: &mut ProbeEndpoint, _: Duration| -> io::Result<bool> {
                        panic!("prequery output error must prevent input polling")
                    },
                    read_endpoint: |_: &mut ProbeEndpoint, _: &mut [u8]| -> io::Result<usize> {
                        panic!("prequery output error must prevent input reading")
                    },
                },
            );

            assert_eq!(
                result
                    .expect_err("prequery output errors are preserved")
                    .kind(),
                expected_kind
            );
            assert_eq!(write_calls.get(), expected_write_calls);
            assert_eq!(stash, b"preexisting input");
            assert_eq!(drops.get(), 1);
        }
    }

    #[test]
    fn wrong_physical_tty_is_rejected_before_query_and_closed() -> io::Result<()> {
        let (_real_master, real) = open_pty()?;
        let (_wrong_master, wrong) = open_pty()?;
        let real_fd = real.as_raw_fd();
        let wrong_fd = wrong.as_raw_fd();
        // SAFETY: F_GETFL only reads the bridge-owned PTY descriptor.
        let original_flags = unsafe { libc::fcntl(real_fd, libc::F_GETFL) };
        assert!(original_flags >= 0);
        let opened_endpoint = Cell::new(-1);
        let query_written = Cell::new(false);
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut stash = b"prior".to_vec();

        let result = probe_keyboard_enhancement_with(
            &mut stash,
            deadline,
            ProbeIo {
                now: Instant::now,
                open_endpoint: |_: Duration| {
                    let endpoint = open_probe_endpoint(wrong_fd, deadline)?;
                    opened_endpoint.set(endpoint.as_raw_fd());
                    Ok(endpoint)
                },
                endpoint_matches_real_tty: |endpoint: &mut OwnedFd, _: Duration| {
                    endpoint_matches_real_tty(real_fd, endpoint.as_raw_fd())
                },
                output_is_ready: |_: &mut OwnedFd, _: Duration| -> io::Result<bool> {
                    panic!("wrong tty must not be polled")
                },
                write_endpoint: |_: &mut OwnedFd, _: &[u8]| -> io::Result<usize> {
                    query_written.set(true);
                    panic!("wrong tty must not receive a query")
                },
                input_is_ready: |_: &mut OwnedFd, _: Duration| -> io::Result<bool> {
                    panic!("wrong tty must not be polled")
                },
                read_endpoint: |_: &mut OwnedFd, _: &mut [u8]| -> io::Result<usize> {
                    panic!("wrong tty must not be read")
                },
            },
        );

        assert_eq!(
            result
                .expect_err("a different physical tty is rejected before query")
                .kind(),
            io::ErrorKind::InvalidData
        );
        assert!(!query_written.get());
        assert_eq!(stash, b"prior");
        assert_closed(opened_endpoint.get());
        // SAFETY: F_GETFL only reads the bridge-owned PTY descriptor.
        assert_eq!(
            unsafe { libc::fcntl(real_fd, libc::F_GETFL) },
            original_flags
        );
        Ok(())
    }

    #[test]
    fn owned_physical_endpoint_closes_after_success_error_and_unwind() -> io::Result<()> {
        for termination in [
            ProbeTermination::Success,
            ProbeTermination::Error,
            ProbeTermination::Panic,
        ] {
            let (_master, real) = open_pty()?;
            let real_fd = real.as_raw_fd();
            let opened_endpoint = Cell::new(-1);
            let mut reply = Some(b"\x1b[?0u".to_vec());
            let mut stash = Vec::new();
            let deadline = Instant::now() + Duration::from_secs(2);

            let mut run_probe = || {
                probe_keyboard_enhancement_with(
                    &mut stash,
                    deadline,
                    ProbeIo {
                        now: Instant::now,
                        open_endpoint: |_: Duration| {
                            let endpoint = open_probe_endpoint(real_fd, deadline)?;
                            opened_endpoint.set(endpoint.as_raw_fd());
                            Ok(endpoint)
                        },
                        endpoint_matches_real_tty: |endpoint: &mut OwnedFd, _: Duration| {
                            endpoint_matches_real_tty(real_fd, endpoint.as_raw_fd())
                        },
                        output_is_ready: |_: &mut OwnedFd, _: Duration| Ok(true),
                        write_endpoint: |_: &mut OwnedFd, query: &[u8]| Ok(query.len()),
                        input_is_ready: |_: &mut OwnedFd, _: Duration| match termination {
                            ProbeTermination::Success => Ok(true),
                            ProbeTermination::Error => {
                                Err(io::Error::from(io::ErrorKind::BrokenPipe))
                            }
                            ProbeTermination::Panic => panic!("injected probe unwind"),
                        },
                        read_endpoint: |_: &mut OwnedFd, buffer: &mut [u8]| {
                            let response = reply.take().expect("one scripted flags reply");
                            buffer[..response.len()].copy_from_slice(&response);
                            Ok(response.len())
                        },
                    },
                )
            };

            match termination {
                ProbeTermination::Success => {
                    assert!(run_probe()?);
                    assert!(stash.is_empty());
                }
                ProbeTermination::Error => {
                    assert_eq!(
                        run_probe()
                            .expect_err("injected poll error is retained")
                            .kind(),
                        io::ErrorKind::BrokenPipe
                    );
                }
                ProbeTermination::Panic => {
                    assert!(catch_unwind(AssertUnwindSafe(&mut run_probe)).is_err());
                }
            }
            assert_closed(opened_endpoint.get());
        }
        Ok(())
    }

    #[test]
    fn keyboard_probe_accepts_zero_flags_at_every_reply_split_without_da() {
        let response = b"\x1b[?0u";
        for split in 1..response.len() {
            let mut stash = Vec::new();
            let (result, script, drops, caller_flags) = run_probe_with_chunks(
                vec![response[..split].to_vec(), response[split..].to_vec()],
                &mut stash,
            );
            assert!(detected(result));
            assert!(stash.is_empty());
            assert_eq!(script.query, QUERY.as_bytes());
            assert_eq!(script.input_polls, 2);
            assert_eq!(drops, 1);
            assert_eq!(caller_flags, 0x1234);
        }
    }

    #[test]
    fn keyboard_probe_preserves_alt_paste_and_interleaved_bytes() {
        let before = b"\x1b\x1bx\x1b[200~paste:\x1b[?0u:\x1b[201~before";
        let response = b"\x1b[?5u";
        let after = b"after\x1b[?x";
        let input = [before.as_slice(), response.as_slice(), after.as_slice()].concat();
        let mut stash = vec![0x1b];

        let (result, script, drops, caller_flags) = run_probe_with_chunks(vec![input], &mut stash);

        assert!(detected(result));
        assert_eq!(
            stash,
            [b"\x1b".as_slice(), before.as_slice(), after.as_slice()].concat()
        );
        assert_eq!(script.query, QUERY.as_bytes());
        assert_eq!(script.reads, 1);
        assert_eq!(drops, 1);
        assert_eq!(caller_flags, 0x1234);
    }

    #[test]
    fn keyboard_probe_replays_invalid_and_partial_csi_on_timeout() {
        let input = b"ordinary\x1b[?256u\x1b[?x\x1b[?12";
        let mut stash = b"prior".to_vec();
        let (result, script, drops, caller_flags) =
            run_probe_with_chunks(vec![input.to_vec()], &mut stash);

        assert!(!result.expect("no valid flags response is not confirmed"));
        assert_eq!(stash, [b"prior".as_slice(), input.as_slice()].concat());
        assert_eq!(script.query, QUERY.as_bytes());
        assert_eq!(drops, 1);
        assert_eq!(caller_flags, 0x1234);
    }

    #[test]
    fn keyboard_probe_replays_a_flags_reply_read_after_deadline() {
        let base = Instant::now();
        let deadline = base + Duration::from_secs(2);
        let elapsed = Rc::new(Cell::new(Duration::ZERO));
        let read_elapsed = Rc::clone(&elapsed);
        let mut response = Some(b"\x1b[?0u".to_vec());
        let drops = Rc::new(Cell::new(0));
        let open_drops = Rc::clone(&drops);
        let mut stash = Vec::new();
        let mut query = Vec::new();
        let result = probe_keyboard_enhancement_with(
            &mut stash,
            deadline,
            ProbeIo {
                now: move || base + elapsed.get(),
                open_endpoint: move |_: Duration| {
                    Ok(ProbeEndpoint {
                        drops: Rc::clone(&open_drops),
                        nonblocking: true,
                    })
                },
                endpoint_matches_real_tty: |endpoint: &mut ProbeEndpoint, _: Duration| {
                    Ok(endpoint.nonblocking)
                },
                output_is_ready: |_: &mut ProbeEndpoint, _: Duration| Ok(true),
                write_endpoint: |_: &mut ProbeEndpoint, bytes: &[u8]| {
                    query.extend_from_slice(bytes);
                    Ok(bytes.len())
                },
                input_is_ready: |_: &mut ProbeEndpoint, _: Duration| Ok(true),
                read_endpoint: move |_: &mut ProbeEndpoint, buffer: &mut [u8]| {
                    let bytes = response.take().expect("the scripted response is read once");
                    buffer[..bytes.len()].copy_from_slice(&bytes);
                    read_elapsed.set(Duration::from_millis(2_001));
                    Ok(bytes.len())
                },
            },
        );

        assert!(!result.expect("late response is not support evidence"));
        assert_eq!(stash, b"\x1b[?0u");
        assert_eq!(query, QUERY.as_bytes());
        assert_eq!(drops.get(), 1);
    }

    #[test]
    fn keyboard_probe_keeps_query_write_retries_inside_the_absolute_deadline() {
        let base = Instant::now();
        let deadline = base + Duration::from_secs(2);
        let elapsed = Rc::new(Cell::new(Duration::ZERO));
        let drops = Rc::new(Cell::new(0));
        let caller_flags = Rc::new(Cell::new(0x1234));
        let timeouts = RefCell::new(Vec::new());
        let open_calls = Cell::new(0);
        let validate_calls = Cell::new(0);
        let output_polls = Cell::new(0);
        let write_calls = Cell::new(0);
        let input_polls = Cell::new(0);
        let mut query = Vec::new();
        let mut stash = Vec::new();

        let open_drops = Rc::clone(&drops);
        let open_timeouts = &timeouts;
        let open_elapsed = Rc::clone(&elapsed);
        let open_calls_seen = &open_calls;
        let validate_elapsed = Rc::clone(&elapsed);
        let validate_calls_seen = &validate_calls;
        let validate_timeouts = &timeouts;
        let output_elapsed = Rc::clone(&elapsed);
        let output_calls_seen = &output_polls;
        let output_timeouts = &timeouts;
        let write_elapsed = Rc::clone(&elapsed);
        let write_calls_seen = &write_calls;
        let write_timeouts = &timeouts;
        let input_elapsed = Rc::clone(&elapsed);
        let input_calls_seen = &input_polls;
        let input_timeouts = &timeouts;
        let flags_seen = Rc::clone(&caller_flags);

        let result = probe_keyboard_enhancement_with(
            &mut stash,
            deadline,
            ProbeIo {
                now: move || base + elapsed.get(),
                open_endpoint: move |remaining: Duration| {
                    open_timeouts.borrow_mut().push(remaining);
                    let call = open_calls_seen.get();
                    open_calls_seen.set(call + 1);
                    open_elapsed.set(open_elapsed.get() + Duration::from_millis(100));
                    if call == 0 {
                        Err(io::Error::from(io::ErrorKind::Interrupted))
                    } else {
                        Ok(ProbeEndpoint {
                            drops: Rc::clone(&open_drops),
                            nonblocking: true,
                        })
                    }
                },
                endpoint_matches_real_tty:
                    move |endpoint: &mut ProbeEndpoint, remaining: Duration| {
                        validate_timeouts.borrow_mut().push(remaining);
                        let call = validate_calls_seen.get();
                        validate_calls_seen.set(call + 1);
                        validate_elapsed.set(validate_elapsed.get() + Duration::from_millis(100));
                        assert!(endpoint.nonblocking);
                        assert_eq!(flags_seen.get(), 0x1234);
                        if call == 0 {
                            Err(io::Error::from(io::ErrorKind::Interrupted))
                        } else {
                            Ok(true)
                        }
                    },
                output_is_ready: move |_: &mut ProbeEndpoint, remaining: Duration| {
                    output_timeouts.borrow_mut().push(remaining);
                    let call = output_calls_seen.get();
                    output_calls_seen.set(call + 1);
                    output_elapsed.set(output_elapsed.get() + Duration::from_millis(100));
                    match call {
                        0 => Err(io::Error::from(io::ErrorKind::Interrupted)),
                        1 => Ok(false),
                        _ => Ok(true),
                    }
                },
                write_endpoint: |_: &mut ProbeEndpoint, bytes: &[u8]| {
                    write_timeouts
                        .borrow_mut()
                        .push(deadline.saturating_duration_since(base + write_elapsed.get()));
                    let call = write_calls_seen.get();
                    write_calls_seen.set(call + 1);
                    write_elapsed.set(write_elapsed.get() + Duration::from_millis(100));
                    match call {
                        0 => Err(io::Error::from(io::ErrorKind::WouldBlock)),
                        1 => Err(io::Error::from(io::ErrorKind::Interrupted)),
                        2 => {
                            query.extend_from_slice(&bytes[..2]);
                            Ok(2)
                        }
                        3 => {
                            query.extend_from_slice(bytes);
                            Ok(bytes.len())
                        }
                        _ => panic!("unexpected query write retry"),
                    }
                },
                input_is_ready: move |_: &mut ProbeEndpoint, remaining: Duration| {
                    input_timeouts.borrow_mut().push(remaining);
                    input_calls_seen.set(input_calls_seen.get() + 1);
                    input_elapsed.set(Duration::from_secs(2));
                    Ok(false)
                },
                read_endpoint: |_: &mut ProbeEndpoint, _: &mut [u8]| -> io::Result<usize> {
                    panic!("no read is ready")
                },
            },
        );

        assert!(!result.expect("the timed query reaches its deadline"));
        assert_eq!(open_calls.get(), 2);
        assert_eq!(validate_calls.get(), 2);
        assert_eq!(output_polls.get(), 6);
        assert_eq!(write_calls.get(), 4);
        assert_eq!(input_polls.get(), 1);
        assert!(timeouts.borrow().windows(2).all(|pair| pair[0] > pair[1]));
        assert_eq!(query, QUERY.as_bytes());
        assert_eq!(drops.get(), 1);
        assert_eq!(caller_flags.get(), 0x1234);
        assert!(stash.is_empty());
    }

    #[test]
    fn keyboard_probe_uses_one_deadline_across_poll_interruptions() {
        let base = Instant::now();
        let deadline = base + Duration::from_secs(2);
        let elapsed = Rc::new(Cell::new(Duration::ZERO));
        let poll_elapsed = Rc::clone(&elapsed);
        let poll_timeouts = RefCell::new(Vec::new());
        let reads = Cell::new(0);
        let polls = Cell::new(0);
        let drops = Rc::new(Cell::new(0));
        let open_drops = Rc::clone(&drops);
        let mut stash = Vec::new();
        let timeouts_seen = &poll_timeouts;
        let reads_seen = &reads;
        let polls_seen = &polls;
        let result = probe_keyboard_enhancement_with(
            &mut stash,
            deadline,
            ProbeIo {
                now: move || base + elapsed.get(),
                open_endpoint: move |_: Duration| {
                    Ok(ProbeEndpoint {
                        drops: Rc::clone(&open_drops),
                        nonblocking: true,
                    })
                },
                endpoint_matches_real_tty: |endpoint: &mut ProbeEndpoint, _: Duration| {
                    Ok(endpoint.nonblocking)
                },
                output_is_ready: |_: &mut ProbeEndpoint, _: Duration| Ok(true),
                write_endpoint: |_: &mut ProbeEndpoint, _: &[u8]| Ok(QUERY.len()),
                input_is_ready: move |_: &mut ProbeEndpoint, timeout: Duration| {
                    timeouts_seen.borrow_mut().push(timeout);
                    polls_seen.set(polls_seen.get() + 1);
                    poll_elapsed.set(poll_elapsed.get() + Duration::from_millis(500));
                    Err(io::Error::from(io::ErrorKind::Interrupted))
                },
                read_endpoint: move |_: &mut ProbeEndpoint, _: &mut [u8]| {
                    reads_seen.set(reads_seen.get() + 1);
                    Ok(0)
                },
            },
        );

        assert!(!result.expect("deadline expires after repeated poll EINTR"));
        let timeouts = poll_timeouts.borrow();
        assert_eq!(timeouts.len(), 4);
        assert!(timeouts.windows(2).all(|pair| pair[0] > pair[1]));
        assert_eq!(reads.get(), 0);
        assert_eq!(polls.get(), 4);
        assert_eq!(drops.get(), 1);
        assert!(stash.is_empty());
    }

    #[test]
    fn keyboard_probe_uses_one_deadline_across_read_interruptions() {
        let base = Instant::now();
        let deadline = base + Duration::from_secs(2);
        let elapsed = Rc::new(Cell::new(Duration::ZERO));
        let read_elapsed = Rc::clone(&elapsed);
        let reads = Cell::new(0);
        let polls = Cell::new(0);
        let drops = Rc::new(Cell::new(0));
        let open_drops = Rc::clone(&drops);
        let mut stash = Vec::new();
        let reads_seen = &reads;
        let polls_seen = &polls;
        let result = probe_keyboard_enhancement_with(
            &mut stash,
            deadline,
            ProbeIo {
                now: move || base + elapsed.get(),
                open_endpoint: move |_: Duration| {
                    Ok(ProbeEndpoint {
                        drops: Rc::clone(&open_drops),
                        nonblocking: true,
                    })
                },
                endpoint_matches_real_tty: |endpoint: &mut ProbeEndpoint, _: Duration| {
                    Ok(endpoint.nonblocking)
                },
                output_is_ready: |_: &mut ProbeEndpoint, _: Duration| Ok(true),
                write_endpoint: |_: &mut ProbeEndpoint, _: &[u8]| Ok(QUERY.len()),
                input_is_ready: move |_: &mut ProbeEndpoint, _: Duration| {
                    polls_seen.set(polls_seen.get() + 1);
                    Ok(true)
                },
                read_endpoint: move |_: &mut ProbeEndpoint, _: &mut [u8]| {
                    let call = reads_seen.get();
                    reads_seen.set(call + 1);
                    match call {
                        0 => {
                            read_elapsed.set(read_elapsed.get() + Duration::from_millis(100));
                            Err(io::Error::from(io::ErrorKind::WouldBlock))
                        }
                        1..=3 => {
                            read_elapsed.set(read_elapsed.get() + Duration::from_millis(500));
                            Err(io::Error::from(io::ErrorKind::Interrupted))
                        }
                        4 => {
                            read_elapsed.set(read_elapsed.get() + Duration::from_millis(400));
                            Err(io::Error::from(io::ErrorKind::Interrupted))
                        }
                        _ => panic!("unexpected read retry"),
                    }
                },
            },
        );

        assert!(!result.expect("deadline expires after one WouldBlock and read EINTR"));
        assert_eq!(reads.get(), 5);
        assert_eq!(polls.get(), 5);
        assert_eq!(drops.get(), 1);
        assert!(stash.is_empty());
    }

    #[test]
    fn keyboard_probe_replays_stale_replies_and_candidates_crossing_the_fresh_boundary() {
        for stale in [b"\x1b[?0u".as_slice(), b"old\x1b[?0".as_slice()] {
            let fresh = if stale.ends_with(b"\x1b[?0") {
                b"u".as_slice()
            } else {
                b"ordinary".as_slice()
            };
            let mut stash = stale.to_vec();
            let (result, _, _, _) = run_probe_with_chunks(vec![fresh.to_vec()], &mut stash);
            assert!(!result.expect("stale reply is not current query evidence"));
            assert_eq!(stash, [stale, fresh].concat());
        }
    }

    #[test]
    fn keyboard_probe_keeps_a_complete_stale_reply_and_consumes_only_fresh_reply() {
        let stale = b"\x1b[?0u";
        let fresh = b"key\x1b[?5u";
        let mut stash = stale.to_vec();
        let (result, _, _, _) = run_probe_with_chunks(vec![fresh.to_vec()], &mut stash);

        assert!(detected(result));
        assert_eq!(stash, [stale.as_slice(), b"key".as_slice()].concat());
    }

    #[test]
    fn keyboard_probe_seeds_an_open_paste_from_the_entire_stale_stash() {
        let stale = b"\x1b[200~prefix";
        let fresh = b"\x1b[?0u\x1b[201~suffix";
        let mut stash = stale.to_vec();
        let (result, _, _, _) = run_probe_with_chunks(vec![fresh.to_vec()], &mut stash);

        assert!(!result.expect("paste payload is not a keyboard flags reply"));
        assert_eq!(stash, [stale.as_slice(), fresh.as_slice()].concat());
    }

    #[test]
    fn keyboard_probe_tracks_every_paste_opener_split_across_the_stash_boundary() {
        let opener = b"\x1b[200~";
        let payload_and_end = b"\x1b[?0u\x1b[201~suffix";
        for split in 1..opener.len() {
            let stale = &opener[..split];
            let fresh = [&opener[split..], payload_and_end].concat();
            let mut stash = stale.to_vec();
            let (result, _, _, _) = run_probe_with_chunks(vec![fresh.clone()], &mut stash);

            assert!(!result.expect("flags-shaped bytes inside split paste are ignored"));
            assert_eq!(stash, [stale, fresh.as_slice()].concat());
        }
    }

    #[test]
    fn keyboard_probe_ignores_split_flags_payload_inside_paste_then_accepts_fresh_reply() {
        let stale = b"\x1b[200~prefix";
        let chunks = vec![
            b"\x1b[?0".to_vec(),
            b"u\x1b[201".to_vec(),
            b"~suffix\x1b[?0".to_vec(),
            b"u".to_vec(),
        ];
        let expected_replay = [
            stale.as_slice(),
            b"\x1b[?0".as_slice(),
            b"u\x1b[201".as_slice(),
            b"~suffix".as_slice(),
        ]
        .concat();
        let mut stash = stale.to_vec();
        let (result, _, _, _) = run_probe_with_chunks(chunks, &mut stash);

        assert!(detected(result));
        assert_eq!(stash, expected_replay);
    }

    #[test]
    fn keyboard_probe_expires_before_any_endpoint_setup_syscall() {
        let deadline = Instant::now() + Duration::from_secs(2);
        let opens = Rc::new(Cell::new(0));
        let observed_opens = Rc::clone(&opens);
        let mut stash = b"preserved".to_vec();
        let result = probe_keyboard_enhancement_with(
            &mut stash,
            deadline,
            ProbeIo {
                now: move || deadline,
                open_endpoint: |_: Duration| -> io::Result<ProbeEndpoint> {
                    observed_opens.set(observed_opens.get() + 1);
                    panic!("endpoint open must not run after deadline")
                },
                endpoint_matches_real_tty: |_: &mut ProbeEndpoint,
                                            _: Duration|
                 -> io::Result<bool> {
                    panic!("endpoint validation must not run")
                },
                output_is_ready: |_: &mut ProbeEndpoint, _: Duration| -> io::Result<bool> {
                    panic!("output poll must not run")
                },
                write_endpoint: |_: &mut ProbeEndpoint, _: &[u8]| -> io::Result<usize> {
                    panic!("query write must not run")
                },
                input_is_ready: |_: &mut ProbeEndpoint, _: Duration| -> io::Result<bool> {
                    panic!("input poll must not run")
                },
                read_endpoint: |_: &mut ProbeEndpoint, _: &mut [u8]| -> io::Result<usize> {
                    panic!("tty read must not run")
                },
            },
        );

        assert_eq!(
            result
                .expect_err("expired setup is returned as a timed-out probe")
                .kind(),
            io::ErrorKind::TimedOut
        );
        assert_eq!(opens.get(), 0);
        assert_eq!(stash, b"preserved");

        let base = Instant::now();
        let deadline = base + Duration::from_secs(2);
        let mut stash = b"still-preserved".to_vec();
        let result = probe_keyboard_enhancement_with(
            &mut stash,
            deadline,
            ProbeIo {
                now: move || base,
                open_endpoint: |_: Duration| -> io::Result<ProbeEndpoint> {
                    Err(io::Error::from(io::ErrorKind::NotFound))
                },
                endpoint_matches_real_tty: |_: &mut ProbeEndpoint,
                                            _: Duration|
                 -> io::Result<bool> {
                    panic!("unopened endpoint cannot be validated")
                },
                output_is_ready: |_: &mut ProbeEndpoint, _: Duration| -> io::Result<bool> {
                    panic!("unopened endpoint cannot be polled")
                },
                write_endpoint: |_: &mut ProbeEndpoint, _: &[u8]| -> io::Result<usize> {
                    panic!("unopened endpoint cannot be written")
                },
                input_is_ready: |_: &mut ProbeEndpoint, _: Duration| -> io::Result<bool> {
                    panic!("unopened endpoint cannot be polled")
                },
                read_endpoint: |_: &mut ProbeEndpoint, _: &mut [u8]| -> io::Result<usize> {
                    panic!("unopened endpoint cannot be read")
                },
            },
        );
        assert_eq!(
            result
                .expect_err("endpoint setup error is preserved")
                .kind(),
            io::ErrorKind::NotFound
        );
        assert_eq!(stash, b"still-preserved");
    }

    #[test]
    fn keyboard_probe_restores_captured_bytes_on_error_and_panic() {
        let base = Instant::now();
        let deadline = base + Duration::from_secs(2);
        let drops = Rc::new(Cell::new(0));
        let open_drops = Rc::clone(&drops);
        let mut stash = b"old".to_vec();
        let mismatch = probe_keyboard_enhancement_with(
            &mut stash,
            deadline,
            ProbeIo {
                now: move || base,
                open_endpoint: move |_: Duration| {
                    Ok(ProbeEndpoint {
                        drops: Rc::clone(&open_drops),
                        nonblocking: true,
                    })
                },
                endpoint_matches_real_tty: |_: &mut ProbeEndpoint, _: Duration| Ok(false),
                output_is_ready: |_: &mut ProbeEndpoint, _: Duration| -> io::Result<bool> {
                    panic!("unverified endpoint must not be polled")
                },
                write_endpoint: |_: &mut ProbeEndpoint, _: &[u8]| -> io::Result<usize> {
                    panic!("unverified endpoint must not receive a query")
                },
                input_is_ready: |_: &mut ProbeEndpoint, _: Duration| -> io::Result<bool> {
                    panic!("unverified endpoint must not be read")
                },
                read_endpoint: |_: &mut ProbeEndpoint, _: &mut [u8]| -> io::Result<usize> {
                    panic!("unverified endpoint must not be read")
                },
            },
        );
        assert_eq!(
            mismatch
                .expect_err("mismatched endpoint is a probe error")
                .kind(),
            io::ErrorKind::InvalidData
        );
        assert_eq!(drops.get(), 1);
        assert_eq!(stash, b"old");

        let error_drops = Rc::new(Cell::new(0));
        let open_error_drops = Rc::clone(&error_drops);
        let caller_flags = Rc::new(Cell::new(0x1234));
        let flags_seen = Rc::clone(&caller_flags);
        let input = b"prefix\x1b[?".to_vec();
        let mut error_stash = b"existing".to_vec();
        let polls = Cell::new(0);
        let error_base = Instant::now();
        let error_deadline = error_base + Duration::from_secs(2);
        let error = probe_keyboard_enhancement_with(
            &mut error_stash,
            error_deadline,
            ProbeIo {
                now: move || error_base,
                open_endpoint: move |_: Duration| {
                    Ok(ProbeEndpoint {
                        drops: Rc::clone(&open_error_drops),
                        nonblocking: true,
                    })
                },
                endpoint_matches_real_tty: move |endpoint: &mut ProbeEndpoint, _: Duration| {
                    assert!(endpoint.nonblocking);
                    assert_eq!(flags_seen.get(), 0x1234);
                    Ok(true)
                },
                output_is_ready: |_: &mut ProbeEndpoint, _: Duration| Ok(true),
                write_endpoint: |_: &mut ProbeEndpoint, bytes: &[u8]| Ok(bytes.len()),
                input_is_ready: |_: &mut ProbeEndpoint, _: Duration| {
                    let call = polls.get();
                    polls.set(call + 1);
                    if call == 0 {
                        Ok(true)
                    } else {
                        Err(io::Error::from(io::ErrorKind::BrokenPipe))
                    }
                },
                read_endpoint: {
                    let input = input.clone();
                    let mut input = Some(input);
                    move |_: &mut ProbeEndpoint, buffer: &mut [u8]| {
                        let bytes = input.take().expect("one scripted read");
                        buffer[..bytes.len()].copy_from_slice(&bytes);
                        Ok(bytes.len())
                    }
                },
            },
        )
        .expect_err("the injected read-side error is preserved");
        assert_eq!(error.kind(), io::ErrorKind::BrokenPipe);
        assert_eq!(
            error_stash,
            [b"existing".as_slice(), input.as_slice()].concat()
        );
        assert_eq!(error_drops.get(), 1);
        assert_eq!(caller_flags.get(), 0x1234);

        let panic_drops = Rc::new(Cell::new(0));
        let open_panic_drops = Rc::clone(&panic_drops);
        let caller_flags = Rc::new(Cell::new(0x1234));
        let flags_seen = Rc::clone(&caller_flags);
        let mut panic_stash = b"old".to_vec();
        let polls = Cell::new(0);
        let panic_base = Instant::now();
        let panic_deadline = panic_base + Duration::from_secs(2);
        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = probe_keyboard_enhancement_with(
                &mut panic_stash,
                panic_deadline,
                ProbeIo {
                    now: move || panic_base,
                    open_endpoint: move |_: Duration| {
                        Ok(ProbeEndpoint {
                            drops: Rc::clone(&open_panic_drops),
                            nonblocking: true,
                        })
                    },
                    endpoint_matches_real_tty: move |endpoint: &mut ProbeEndpoint, _: Duration| {
                        assert!(endpoint.nonblocking);
                        assert_eq!(flags_seen.get(), 0x1234);
                        Ok(true)
                    },
                    output_is_ready: |_: &mut ProbeEndpoint, _: Duration| Ok(true),
                    write_endpoint: |_: &mut ProbeEndpoint, bytes: &[u8]| Ok(bytes.len()),
                    input_is_ready: |_: &mut ProbeEndpoint, _: Duration| {
                        let call = polls.get();
                        polls.set(call + 1);
                        if call == 0 {
                            Ok(true)
                        } else {
                            panic!("injected probe panic")
                        }
                    },
                    read_endpoint: {
                        let input = input.clone();
                        let mut input = Some(input);
                        move |_: &mut ProbeEndpoint, buffer: &mut [u8]| {
                            let bytes = input.take().expect("one scripted read");
                            buffer[..bytes.len()].copy_from_slice(&bytes);
                            Ok(bytes.len())
                        }
                    },
                },
            );
        }));
        assert!(panic.is_err());
        assert_eq!(panic_stash, [b"old".as_slice(), input.as_slice()].concat());
        assert_eq!(panic_drops.get(), 1);
        assert_eq!(caller_flags.get(), 0x1234);
    }
}
