#[derive(Debug, Clone, Copy, PartialEq, Eq)]

pub enum DiscoveryError {
    IndexUnreadable,
}

impl DiscoveryError {
    pub fn label(self) -> &'static str {
        match self {
            DiscoveryError::IndexUnreadable => "instance index unreadable",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]

pub enum EntryDecision {
    PlayIntro,

    JoinActive { running: usize },

    Duplicate,

    Unknown(DiscoveryError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]

pub enum ExitDecision {
    Outro { elapsed_secs: Option<u64> },

    StillInside { remaining: usize },

    AlreadyEnded,

    Unknown(DiscoveryError),
}

#[derive(Debug, Clone, PartialEq, Eq)]

pub struct Arbiter {
    pub discovery: Result<usize, DiscoveryError>,

    pub foreign_claim: bool,

    pending_entry: bool,

    pub entered_at_ms: Option<i64>,

    exit_consumed: bool,
}

impl Arbiter {
    pub fn new(running: usize) -> Self {
        Self {
            discovery: Ok(running),
            foreign_claim: false,
            pending_entry: false,
            entered_at_ms: None,
            exit_consumed: false,
        }
    }

    #[cfg(test)]
    pub fn pending_entry(&self) -> bool {
        self.pending_entry
    }

    pub fn running(&self) -> Result<usize, DiscoveryError> {
        self.discovery
    }

    pub fn set_running(&mut self, n: usize) {
        self.discovery = Ok(n);
    }

    pub fn request_entry(&mut self) -> EntryDecision {
        match self.discovery {
            Err(e) => EntryDecision::Unknown(e),
            Ok(0) => {
                if self.foreign_claim {
                    EntryDecision::Duplicate
                } else {
                    self.pending_entry = true;
                    EntryDecision::PlayIntro
                }
            }
            Ok(n) => EntryDecision::JoinActive { running: n },
        }
    }

    pub fn release_entry(&mut self) {
        self.pending_entry = false;
    }

    pub fn complete_entry(&mut self, now_ms: i64) {
        self.pending_entry = false;
        if self.entered_at_ms.is_none() {
            self.entered_at_ms = Some(now_ms);
        }
    }

    pub fn request_exit(&mut self, now_ms: i64) -> ExitDecision {
        match self.discovery {
            Err(e) => ExitDecision::Unknown(e),
            Ok(n) if n > 0 => ExitDecision::StillInside { remaining: n },
            Ok(_) => {
                if self.exit_consumed {
                    return ExitDecision::AlreadyEnded;
                }
                self.exit_consumed = true;
                let elapsed = self
                    .entered_at_ms
                    .map(|t| (now_ms - t).max(0) as u64 / 1000);
                ExitDecision::Outro {
                    elapsed_secs: elapsed,
                }
            }
        }
    }

    #[cfg(test)]
    pub fn exit_consumed(&self) -> bool {
        self.exit_consumed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_construct_plays_once_and_join_skips() {
        let mut a = Arbiter::new(0);
        assert_eq!(a.request_entry(), EntryDecision::PlayIntro);
        assert!(a.pending_entry());

        assert_eq!(a.request_entry(), EntryDecision::PlayIntro);
        a.complete_entry(10_000);
        assert!(!a.pending_entry());
        a.set_running(1);
        assert_eq!(a.request_entry(), EntryDecision::JoinActive { running: 1 });
    }

    #[test]
    fn foreign_claim_suppresses_duplicate_intro() {
        let mut a = Arbiter::new(0);
        a.foreign_claim = true;
        assert_eq!(a.request_entry(), EntryDecision::Duplicate);
        a.release_entry();
        assert!(!a.pending_entry());
    }

    #[test]
    fn exit_token_has_one_consumer_and_fails_closed() {
        let mut a = Arbiter::new(0);
        a.complete_entry(0);
        a.set_running(1);
        assert_eq!(
            a.request_exit(4_000),
            ExitDecision::StillInside { remaining: 1 }
        );
        a.set_running(0);
        assert_eq!(
            a.request_exit(10_000),
            ExitDecision::Outro {
                elapsed_secs: Some(10)
            }
        );
        assert_eq!(a.request_exit(10_400), ExitDecision::AlreadyEnded);
        let mut b = Arbiter::new(0);
        b.discovery = Err(DiscoveryError::IndexUnreadable);
        assert_eq!(
            b.request_exit(0),
            ExitDecision::Unknown(DiscoveryError::IndexUnreadable)
        );
        assert!(!b.exit_consumed());
    }

    #[test]
    fn missing_entry_time_omits_elapsed() {
        let mut a = Arbiter::new(0);
        assert_eq!(
            a.request_exit(5_000),
            ExitDecision::Outro { elapsed_secs: None }
        );
    }
}
