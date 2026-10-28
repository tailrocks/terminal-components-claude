use std::io::{self, Write};

pub use termrock_core::keys::keyboard::{
    DISAMBIGUATE, FLAGS, KeyboardProtocol, POP, ProtocolState, QUERY, REPORT_ALTERNATE_KEYS,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KeyboardNegotiation {
    state: ProtocolState,
}

impl KeyboardNegotiation {
    pub fn enter(
        &mut self,
        out: &mut impl Write,
        policy: KeyboardProtocol,
        detect: impl FnOnce() -> bool,
    ) -> io::Result<bool> {
        let mut next = self.state;
        if let Some(sequence) = next.enter(policy, detect) {
            write_sequence(out, &sequence, || self.state = next)?;
            return Ok(true);
        }
        self.state = next;
        Ok(false)
    }

    pub fn leave(&mut self, out: &mut impl Write) -> io::Result<bool> {
        let mut next = self.state;
        match next.leave() {
            Some(pop) => {
                write_sequence(out, pop, || self.state = next)?;
                Ok(true)
            }
            None => {
                self.state = next;
                Ok(false)
            }
        }
    }

    pub fn resume(&mut self, out: &mut impl Write) -> io::Result<bool> {
        let mut next = self.state;
        match next.resume() {
            Some(sequence) => {
                write_sequence(out, &sequence, || self.state = next)?;
                Ok(true)
            }
            None => {
                self.state = next;
                Ok(false)
            }
        }
    }

    pub const fn is_pushed(&self) -> bool {
        self.state.pushed
    }

    pub const fn is_enabled(&self) -> bool {
        self.state.enabled
    }

    pub const fn state(&self) -> ProtocolState {
        self.state
    }
}

fn write_sequence(
    out: &mut impl Write,
    sequence: &str,
    commit_after_write: impl FnOnce(),
) -> io::Result<()> {
    out.write_all(sequence.as_bytes())?;
    commit_after_write();
    out.flush()
}
