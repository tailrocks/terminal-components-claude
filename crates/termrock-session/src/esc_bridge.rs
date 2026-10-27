use std::io::{self, ErrorKind};
use std::ops::Range;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use ratatui_crossterm::crossterm::event::{
    Event, KeyCode, KeyEventKind, KeyModifiers, poll as crossterm_poll,
};
use termrock_core::keys::keyboard::{QUERY, parse_flags_reply};

#[derive(Default)]
struct InputState {
    stash: Vec<u8>,
    framing: InputFraming,
    split_prefix_pending: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum InputFraming {
    #[default]
    Ground,
    Escape,
    Utf8 {
        remaining: u8,
    },
    AltUtf8 {
        remaining: u8,
    },
    Csi(CsiFraming),
    Ss3,
    Paste {
        close_progress: u8,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CsiFraming {
    Start,
    DoubleBracket,
    Private { length: u8 },
    Numbered { paste_progress: u8 },
    SgrMouse,
    NormalMouse { length: u8 },
}

impl InputFraming {
    fn at_event_boundary(self) -> bool {
        self == Self::Ground
    }

    fn feed(&mut self, byte: u8) {
        *self = match *self {
            Self::Ground => match byte {
                0x1b => Self::Escape,
                0xc0..=0xdf => Self::Utf8 { remaining: 1 },
                0xe0..=0xef => Self::Utf8 { remaining: 2 },
                0xf0..=0xf7 => Self::Utf8 { remaining: 3 },
                _ => Self::Ground,
            },
            Self::Escape => match byte {
                b'[' => Self::Csi(CsiFraming::Start),
                b'O' => Self::Ss3,
                0x1b => Self::Ground,
                0xc0..=0xdf => Self::AltUtf8 { remaining: 1 },
                0xe0..=0xef => Self::AltUtf8 { remaining: 2 },
                0xf0..=0xf7 => Self::AltUtf8 { remaining: 3 },
                _ => Self::Ground,
            },
            Self::Utf8 { remaining } => feed_utf8(remaining, byte, false),
            Self::AltUtf8 { remaining } => feed_utf8(remaining, byte, true),
            Self::Csi(CsiFraming::Start) => match byte {
                b'?' => Self::Csi(CsiFraming::Private { length: 3 }),
                b'<' => Self::Csi(CsiFraming::SgrMouse),
                b'M' => Self::Csi(CsiFraming::NormalMouse { length: 3 }),
                b'0'..=b'9' => Self::Csi(CsiFraming::Numbered {
                    paste_progress: u8::from(byte == b'2'),
                }),

                b'D' | b'C' | b'A' | b'B' | b'H' | b'F' | b'Z' | b'I' | b'O' | b'P' | b'Q'
                | b'S' => Self::Ground,
                b'[' => Self::Csi(CsiFraming::DoubleBracket),

                _ => Self::Ground,
            },
            Self::Csi(CsiFraming::DoubleBracket) => Self::Ground,
            Self::Csi(CsiFraming::Private { length }) => match byte {
                b'c' => Self::Ground,
                b'u' if length >= 4 => Self::Ground,
                _ => Self::Csi(CsiFraming::Private {
                    length: length.saturating_add(1),
                }),
            },
            Self::Csi(CsiFraming::Numbered { paste_progress }) => {
                if paste_progress == 3 && byte == b'~' {
                    Self::Paste { close_progress: 0 }
                } else if (0x40..=0x7e).contains(&byte) {
                    Self::Ground
                } else {
                    Self::Csi(CsiFraming::Numbered {
                        paste_progress: next_paste_start_progress(paste_progress, byte),
                    })
                }
            }
            Self::Csi(CsiFraming::SgrMouse) => match byte {
                b'm' | b'M' => Self::Ground,
                _ => Self::Csi(CsiFraming::SgrMouse),
            },
            Self::Csi(CsiFraming::NormalMouse { length }) => {
                if length >= 5 {
                    Self::Ground
                } else {
                    Self::Csi(CsiFraming::NormalMouse {
                        length: length.saturating_add(1),
                    })
                }
            }
            Self::Ss3 => Self::Ground,
            Self::Paste { close_progress } => {
                let next = next_paste_end_progress(close_progress, byte);
                if next == BRACKETED_PASTE_END.len() as u8 {
                    Self::Ground
                } else {
                    Self::Paste {
                        close_progress: next,
                    }
                }
            }
        };
    }
}

fn feed_utf8(remaining: u8, byte: u8, alt: bool) -> InputFraming {
    if byte & 0xc0 != 0x80 {
        return InputFraming::Ground;
    }
    if remaining == 1 {
        InputFraming::Ground
    } else if alt {
        InputFraming::AltUtf8 {
            remaining: remaining - 1,
        }
    } else {
        InputFraming::Utf8 {
            remaining: remaining - 1,
        }
    }
}

fn next_paste_start_progress(progress: u8, byte: u8) -> u8 {
    match (progress, byte) {
        (1, b'0') => 2,
        (2, b'0') => 3,
        _ => 0,
    }
}

fn next_paste_end_progress(progress: u8, byte: u8) -> u8 {
    if BRACKETED_PASTE_END.get(usize::from(progress)) == Some(&byte) {
        progress + 1
    } else {
        u8::from(byte == BRACKETED_PASTE_END[0])
    }
}

impl InputState {
    fn acknowledge_split_prefix_consumed(&mut self, event: &Event) {
        let is_standalone_escape = matches!(
            event,
            Event::Key(key)
                if key.code == KeyCode::Esc
                    && key.modifiers == KeyModifiers::NONE
                    && key.kind == KeyEventKind::Press
        );
        if is_standalone_escape && self.framing == InputFraming::Escape {
            self.framing = InputFraming::Ground;
            self.split_prefix_pending = false;
        }
    }

    fn prepare_write(&mut self, incoming: &[u8]) -> Option<Vec<u8>> {
        self.stash.extend_from_slice(incoming);
        if self.split_prefix_pending || self.stash.is_empty() {
            return None;
        }

        let bytes = std::mem::take(&mut self.stash);
        for (index, byte) in bytes.iter().copied().enumerate() {
            if self.framing.at_event_boundary() && byte == 0x1b {
                if bytes.get(index + 1) == Some(&0x1b) {
                    self.framing.feed(byte);
                    self.stash.extend_from_slice(&bytes[index + 1..]);
                    self.split_prefix_pending = true;
                    return Some(bytes[..=index].to_vec());
                }
                if index + 1 == bytes.len() {
                    self.stash.push(byte);
                    return (!bytes[..index].is_empty()).then(|| bytes[..index].to_vec());
                }
            }
            self.framing.feed(byte);
        }
        Some(bytes)
    }

    fn take_held_escape_for_flush(&mut self) -> bool {
        if self.split_prefix_pending || self.stash != [0x1b] {
            return false;
        }
        self.stash.clear();
        self.framing.feed(0x1b);

        self.split_prefix_pending = true;
        true
    }
}

struct Bridge {
    real: OwnedFd,
    master: OwnedFd,
    input: InputState,

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
            &mut active.input.stash,
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

    rollback.stdin_touched = true;
    replace_stdin_fn(slave.as_raw_fd())?;

    rollback.armed = false;
    drop(rollback);
    *bridge() = Some(Bridge {
        real,
        master,
        input: InputState::default(),
        original,
    });
    Ok(())
}

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

fn take_ready_stash() -> Option<Vec<u8>> {
    let mut guard = bridge();
    let bridge = guard.as_mut()?;
    bridge.input.prepare_write(&[])
}

fn queue_bytes(bytes: &[u8]) -> io::Result<()> {
    let output = {
        let mut guard = bridge();
        let Some(bridge) = guard.as_mut() else {
            return Ok(());
        };
        bridge.input.prepare_write(bytes)
    };
    if let Some(output) = output {
        write_master(&output)?;
    }
    Ok(())
}

fn flush_held_esc() -> io::Result<()> {
    let held = {
        let mut guard = bridge();
        let Some(bridge) = guard.as_mut() else {
            return Ok(());
        };
        bridge.input.take_held_escape_for_flush()
    };
    if held {
        write_master(&[0x1b])?;
    }
    Ok(())
}

pub(crate) fn acknowledge_split_prefix_consumed(event: &Event) {
    let mut guard = bridge();
    if let Some(bridge) = guard.as_mut() {
        bridge.input.acknowledge_split_prefix_consumed(event);
    }
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

// SAFETY: `fd` is open and `dup` returns a new descriptor on success.

// SAFETY: `dup` returned a new descriptor owned by this function.

// SAFETY: `fstat` initializes each value for the supplied open descriptor.

// SAFETY: `fstat` initializes each value for the supplied open descriptor.

// SAFETY: F_GETFD only inspects this descriptor number.

// SAFETY: F_GETFL only reads the open PTY descriptor's status flags.

// SAFETY: F_GETFL only observes this independently opened endpoint.

// SAFETY: F_GETFL only reads the bridge-owned PTY descriptor.

// SAFETY: F_GETFL only reads the bridge-owned PTY descriptor.

// SAFETY: F_GETFL only reads the bridge-owned PTY descriptor.
