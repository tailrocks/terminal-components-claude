//! Disposable planning feasibility probe, not a Terminal Components implementation.
#![forbid(unsafe_code)]

use std::io::{self, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    DisableLineWrap, EnableLineWrap, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
    enable_raw_mode,
};
use signal_hook::consts::SIGTSTP;
use signal_hook::{SigId, flag, low_level};

const QUANTUM_MS: u64 = 100;

// Handlers outlive individual sessions. The supported process starts with the
// default SIGTSTP disposition and installs no competing SIGTSTP handler.
struct Broker {
    inactive: Arc<AtomicBool>,
    pending: Arc<AtomicBool>,
    leased: AtomicBool,
    handlers: [SigId; 2],
}

impl Broker {
    fn install() -> io::Result<Self> {
        let inactive = Arc::new(AtomicBool::new(true));
        let pending = Arc::new(AtomicBool::new(false));
        let default = flag::register_conditional_default(SIGTSTP, Arc::clone(&inactive))?;
        // If the second registration fails, the inactive default emulation
        // remains valid. No terminal mode has been acquired yet.
        let record = flag::register(SIGTSTP, Arc::clone(&pending))?;
        Ok(Self {
            inactive,
            pending,
            leased: AtomicBool::new(false),
            handlers: [default, record],
        })
    }

    fn lease(&self) -> io::Result<Session<'_>> {
        self.leased
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .map_err(|_| io::Error::other("overlapping terminal session"))?;
        self.pending.store(false, Ordering::SeqCst);
        // Switch to deferred handling before any terminal mode can be changed.
        self.inactive.store(false, Ordering::SeqCst);
        Ok(Session {
            broker: self,
            acquired: 0,
        })
    }
}

struct Session<'a> {
    broker: &'a Broker,
    acquired: u8,
}

impl Session<'_> {
    fn enter(&mut self, fail_after: Option<u8>) -> io::Result<()> {
        for step in 1..=5 {
            // Mark before the operation so partial writes are also cleaned up.
            self.acquired = step;
            match step {
                1 => enable_raw_mode()?,
                2 => execute!(io::stdout(), EnterAlternateScreen)?,
                3 => execute!(io::stdout(), event::EnableMouseCapture)?,
                4 => execute!(io::stdout(), event::EnableBracketedPaste)?,
                5 => execute!(io::stdout(), DisableLineWrap)?,
                _ => unreachable!("the setup range has five steps"),
            }
            if fail_after == Some(step) {
                return Err(io::Error::other("injected setup failure"));
            }
        }
        Ok(())
    }

    fn restore(&mut self) -> io::Result<()> {
        let mut first_error = None;
        // Attempt every reverse step even if an earlier output operation fails.
        while self.acquired > 0 {
            let step = self.acquired;
            let result = match step {
                5 => execute!(io::stdout(), EnableLineWrap),
                4 => execute!(io::stdout(), event::DisableBracketedPaste),
                3 => execute!(io::stdout(), event::DisableMouseCapture),
                2 => execute!(io::stdout(), LeaveAlternateScreen),
                1 => disable_raw_mode(),
                _ => unreachable!("only completed setup steps are restored"),
            };
            if first_error.is_none() {
                first_error = result.err();
            }
            self.acquired -= 1;
        }
        match first_error {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    fn suspend(&mut self, dirty_stop: bool) -> io::Result<()> {
        if !dirty_stop {
            self.restore()?;
        }
        low_level::emulate_default_handler(SIGTSTP)?;
        // Requests while already stopping/stopped coalesce into this stop.
        self.broker.pending.store(false, Ordering::SeqCst);
        if dirty_stop {
            self.restore()?;
        }
        self.enter(None)?;
        let (cols, rows) = crossterm::terminal::size()?;
        marker(&format!("RESUMED {cols} {rows}"))?;
        Ok(())
    }
}

impl Drop for Session<'_> {
    fn drop(&mut self) {
        let _ = self.restore();
        // Never enable default stopping until terminal cleanup was attempted.
        self.broker.inactive.store(true, Ordering::SeqCst);
        self.broker.leased.store(false, Ordering::SeqCst);
    }
}

fn wait_budget(now_ms: u64, deadline_ms: Option<u64>) -> Duration {
    Duration::from_millis(
        deadline_ms
            .map(|deadline| deadline.saturating_sub(now_ms))
            .unwrap_or(QUANTUM_MS)
            .min(QUANTUM_MS),
    )
}

fn marker(message: &str) -> io::Result<()> {
    let mut output = io::stdout().lock();
    writeln!(output, "\r\n@@{message}")?;
    output.flush()
}

fn continue_line() -> io::Result<()> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    if input.trim() != "continue" {
        return Err(io::Error::other("expected continue"));
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let mode = std::env::args().nth(1).unwrap_or_default();
    // Test supervisor first establishes the foreground group. Do not race
    // terminal setup against that harness-only handoff.
    marker("AWAIT_START")?;
    continue_line()?;
    let broker = Broker::install()?;
    {
        let mut partial = broker.lease()?;
        let result = partial.enter(Some(4));
        if result.is_ok() {
            return Err(io::Error::other("failure injection did not execute"));
        }
        if broker.lease().is_ok() {
            return Err(io::Error::other("overlapping lease admitted"));
        }
    }
    marker("FAILED_SETUP_CLEAN")?;
    continue_line()?;

    for cycle in 1..=2 {
        let mut session = broker.lease()?;
        session.enter(None)?;
        marker(&format!("READY {cycle}"))?;
        let mut wakes = 0_u64;
        let mut logical_ticks = 0_u64;
        loop {
            if broker.pending.swap(false, Ordering::SeqCst) {
                session.suspend(mode == "dirty-stop")?;
            }
            let input_ready = if mode == "block-read" {
                true
            } else {
                event::poll(wait_budget(0, None))?
            };
            if input_ready {
                if let Event::Key(key) = event::read()?
                    && key.code == KeyCode::Char('n')
                {
                    break;
                }
            } else {
                wakes += 1;
                if mode == "idle-tick" {
                    logical_ticks += 1;
                }
            }
        }
        drop(session);
        if mode == "unregister" {
            for handler in broker.handlers {
                low_level::unregister(handler);
            }
        }
        marker(&format!(
            "INACTIVE {cycle} wakes={wakes} ticks={logical_ticks} settle=0"
        ))?;
        continue_line()?;
    }
    marker("EXIT")
}

#[cfg(test)]
mod tests {
    use super::wait_budget;
    use std::time::Duration;

    #[test]
    fn exact_due_overdue_and_service_bounds() {
        let expected = [
            (99, 0),
            (100, 0),
            (101, 1),
            (199, 99),
            (200, 100),
            (201, 100),
        ];
        for (deadline, wait) in expected {
            assert_eq!(
                wait_budget(100, Some(deadline)),
                Duration::from_millis(wait)
            );
        }
        assert_eq!(wait_budget(100, None), Duration::from_millis(100));
    }

    #[test]
    fn admitted_input_does_not_rebase_the_original_deadline() {
        let deadline = Some(101);
        for _ in 0..1000 {
            assert_eq!(wait_budget(100, deadline), Duration::from_millis(1));
        }
        assert_eq!(wait_budget(101, deadline), Duration::ZERO);
        assert_eq!(wait_budget(102, deadline), Duration::ZERO);
    }
}
