//! Keep a second ESC when one read contains Esc and then Alt+key.
//!
//! Crossterm 0.29 parses `\x1b\x1b` as one Esc and clears its buffer, so the
//! Alt prefix is gone and the next byte is a plain character. TablePro's
//! explain-analyze chord is Esc, then Alt+x. A slow 160-column frame delivers
//! those bytes in one read (`\x1b\x1bx`). This bridge owns the real tty and
//! writes the first Esc to a side pty before the second Esc, so Crossterm
//! sees two reads.

use std::io::{self, ErrorKind};
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
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
    use std::io;
    use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
    use std::rc::Rc;

    use super::{
        dup_stdin, install, install_with, installed, make_raw, open_pty, open_raw_pty,
        read_termios, replace_stdin, restore_stdin, second_esc_index, set_cloexec, write_termios,
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
}
