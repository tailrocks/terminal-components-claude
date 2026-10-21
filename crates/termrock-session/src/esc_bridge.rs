//! Keep a second ESC when one read contains Esc and then Alt+key.
//!
//! Crossterm 0.29 parses `\x1b\x1b` as one Esc and clears its buffer, so the
//! Alt prefix is gone and the next byte is a plain character. TablePro's
//! explain-analyze chord is Esc, then Alt+x. A slow 160-column frame delivers
//! those bytes in one read (`\x1b\x1bx`). This bridge owns the real tty and
//! writes the first Esc to a side pty before the second Esc, so Crossterm
//! sees two reads.

use std::io::{self, ErrorKind};
use std::os::fd::RawFd;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use ratatui_crossterm::crossterm::event::poll as crossterm_poll;

/// Index of the second ESC in the first `\x1b\x1b` pair.
pub(crate) fn second_esc_index(bytes: &[u8]) -> Option<usize> {
    bytes
        .windows(2)
        .position(|pair| pair == [0x1b, 0x1b])
        .map(|index| index + 1)
}

struct Bridge {
    real: RawFd,
    master: RawFd,
    stash: Vec<u8>,
    /// Termios of the real tty before this bridge switched it to raw mode.
    original: libc::termios,
}

static BRIDGE: Mutex<Option<Bridge>> = Mutex::new(None);

fn bridge() -> std::sync::MutexGuard<'static, Option<Bridge>> {
    BRIDGE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub(crate) fn installed() -> bool {
    bridge().is_some()
}

/// Point stdin at a side pty and keep the real tty for reading.
///
/// Call this before any `crossterm::event::poll` or `read`. Those bind the
/// process-wide reader to stdin. Swapping stdin afterwards leaves that reader
/// on the wrong descriptor, so application keys never arrive.
pub(crate) fn install() -> io::Result<()> {
    if bridge().is_some() {
        return Ok(());
    }
    let real = dup_stdin()?;
    let original = match read_termios(real) {
        Ok(attrs) => attrs,
        Err(err) => {
            close_fd(real);
            return Err(err);
        }
    };
    if let Err(err) = make_raw(real) {
        close_fd(real);
        return Err(err);
    }
    let (master, slave) = match open_raw_pty() {
        Ok(pair) => pair,
        Err(err) => {
            let _ = write_termios(real, &original);
            close_fd(real);
            return Err(err);
        }
    };
    // SAFETY: `slave` is the openpty slave. Later Crossterm reads see only
    // the bytes this bridge writes to `master`.
    let rc = unsafe { libc::dup2(slave, libc::STDIN_FILENO) };
    if rc < 0 {
        let err = io::Error::last_os_error();
        close_fd(master);
        close_fd(slave);
        let _ = write_termios(real, &original);
        close_fd(real);
        return Err(err);
    }
    close_fd(slave);
    set_cloexec(real)?;
    set_cloexec(master)?;
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
    let rc = unsafe { libc::dup2(bridge.real, libc::STDIN_FILENO) };
    if rc >= 0 {
        let _ = write_termios(libc::STDIN_FILENO, &bridge.original);
    }
    close_fd(bridge.real);
    close_fd(bridge.master);
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
                    return Ok(crossterm_poll(Duration::ZERO)?);
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
        .map(|bridge| bridge.real)
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
        .map(|bridge| bridge.real)
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
        .map(|bridge| bridge.master)
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

fn dup_stdin() -> io::Result<RawFd> {
    // SAFETY: duplicating stdin. The caller closes the result.
    let fd = unsafe { libc::dup(libc::STDIN_FILENO) };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }
    if unsafe { libc::isatty(fd) } != 1 {
        close_fd(fd);
        return Err(io::Error::new(ErrorKind::Unsupported, "stdin is not a tty"));
    }
    Ok(fd)
}

fn open_raw_pty() -> io::Result<(RawFd, RawFd)> {
    let mut master = 0;
    let mut slave = 0;
    // SAFETY: openpty writes the two fds. Null name, termios, and size keep
    // the implementation defaults; the slave is switched to raw mode next.
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
    if let Err(err) = make_raw(slave) {
        close_fd(master);
        close_fd(slave);
        return Err(err);
    }
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

fn close_fd(fd: RawFd) {
    // SAFETY: `fd` is an owned bridge descriptor that is not used again.
    unsafe { libc::close(fd) };
}

#[cfg(test)]
mod tests {
    use super::second_esc_index;

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
}
