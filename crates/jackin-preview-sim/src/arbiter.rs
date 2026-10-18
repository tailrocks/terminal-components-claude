#![doc = r" Construct boundary arbiter: the deterministic, in-memory model of the"]
#![doc = r" cross-process rules that decide when the entry ritual plays and who"]
#![doc = r" gets to play the exit ritual."]
#![doc = r""]
#![doc = r" - A pending entry claim suppresses a duplicate intro while another"]
#![doc = r"   client is already entering the empty Construct."]
#![doc = r" - Remaining-instance discovery returns a typed result; a failure is"]
#![doc = r"   surfaced and the rich outro is withheld (fail closed)."]
#![doc = r" - One exit token exists per Construct; the first consumer plays the"]
#![doc = r"   outro, later requests are told the Construct already ended."]
#![doc = r" - Repeating a message (claim, release, complete, exit) is idempotent."]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[doc = r" Discovery failure modes for remaining-instance lookup."]
pub enum DiscoveryError {
    #[doc = r" The daemon index could not be read."]
    IndexUnreadable,
}

impl DiscoveryError {
    #[doc = r" Human-readable label for the failure."]
    pub fn label(self) -> &'static str {
        match self {
            DiscoveryError::IndexUnreadable => "instance index unreadable",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[doc = r" Outcome of an entry request."]
pub enum EntryDecision {
    #[doc = r" Zero running instances and no other claim: play the intro."]
    PlayIntro,
    #[doc = r" The Construct is active with `running` instances: join without replay."]
    #[doc = r" Join the active Construct with `running` attached instances."]
    JoinActive {
        #[doc = r" Instances currently attached."]
        running: usize,
    },
    #[doc = r" Another client already holds the pending entry claim."]
    Duplicate,
    #[doc = r" Discovery failed: enter without the ritual and report the failure."]
    Unknown(DiscoveryError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[doc = r" Outcome of an exit request."]
pub enum ExitDecision {
    #[doc = r" This client consumed the exit token: play the rich outro."]
    #[doc = r" Rich outro, with the elapsed session time if known."]
    Outro {
        #[doc = r" Elapsed session time, if known."]
        elapsed_secs: Option<u64>,
    },
    #[doc = r" Other instances remain: compact still-inside feedback."]
    #[doc = r" Compact feedback: `remaining` instances are still attached."]
    StillInside {
        #[doc = r" Instances still attached after this exit."]
        remaining: usize,
    },
    #[doc = r" Somebody else already played the outro for this Construct."]
    AlreadyEnded,
    #[doc = r" Discovery failed: fail closed, no rich outro."]
    Unknown(DiscoveryError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[doc = r" Deterministic entry/exit boundary state for one Construct."]
pub struct Arbiter {
    #[doc = r" What remaining-instance discovery reports; scenarios seed it."]
    pub discovery: Result<usize, DiscoveryError>,
    #[doc = r" Another (simulated) client already entering the empty Construct."]
    pub foreign_claim: bool,
    #[doc = r" Whether a client currently holds the pending entry claim."]
    pending_entry: bool,
    #[doc = r" Fixture instant (virtual ms) at which the Construct was entered."]
    pub entered_at_ms: Option<i64>,
    #[doc = r" Whether the single exit token has been consumed."]
    exit_consumed: bool,
}

impl Arbiter {
    #[doc = r" Seed the arbiter with `running` discovered instances."]
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
    #[doc = r" Test probe for the pending entry claim."]
    pub fn pending_entry(&self) -> bool {
        self.pending_entry
    }

    #[doc = r" Current discovered instance count."]
    pub fn running(&self) -> Result<usize, DiscoveryError> {
        self.discovery
    }

    #[doc = r" Override the discovered instance count."]
    pub fn set_running(&mut self, n: usize) {
        self.discovery = Ok(n);
    }

    #[doc = r" Ask to enter. Idempotent: a client that already holds the claim gets"]
    #[doc = r" `PlayIntro` again rather than `Duplicate`."]
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

    #[doc = r" An idle quit before any instance started: drop the claim."]
    pub fn release_entry(&mut self) {
        self.pending_entry = false;
    }

    #[doc = r" The ritual finished or the client joined: the Construct is entered."]
    pub fn complete_entry(&mut self, now_ms: i64) {
        self.pending_entry = false;
        if self.entered_at_ms.is_none() {
            self.entered_at_ms = Some(now_ms);
        }
    }

    #[doc = r" A foreground client is leaving its instance; `discovery` already"]
    #[doc = r" reflects what remains once this instance is gone."]
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
    #[doc = r" Test probe for the exit token."]
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
