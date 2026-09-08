//! The simulated world: one deterministic fixture universe per scenario.
//! Discovery is progressive and per domain — each capability lands at its
//! own virtual-ms mark so early frames honestly show a half-known folder.

use crate::clock::Clock;
use crate::domain::activity::Activity;
use crate::domain::debian::DebianState;
use crate::domain::disk::DiskState;
use crate::domain::docker::DockerState;
use crate::domain::git::GitRepo;
use crate::domain::github::GhState;
use crate::domain::host::Host;
use crate::domain::mise::MiseState;
use crate::domain::pg::PgSession;
use crate::domain::ranking::Memory;
use crate::domain::ssh::SshHost;
use crate::scenario::Scenario;

/// A capability domain discovery can land (or fail) in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Domain {
    Mise,
    Git,
    Ssh,
    Github,
    Docker,
    Pg,
    Disk,
}

impl Domain {
    #[allow(dead_code)] // P2 renders per-domain scanning notes
    pub(crate) const ALL: [Domain; 7] = [
        Domain::Mise,
        Domain::Git,
        Domain::Ssh,
        Domain::Github,
        Domain::Docker,
        Domain::Pg,
        Domain::Disk,
    ];

    #[allow(dead_code)] // P2 renders per-domain scanning notes
    pub(crate) fn label(self) -> &'static str {
        match self {
            Domain::Mise => "mise",
            Domain::Git => "git",
            Domain::Ssh => "ssh",
            Domain::Github => "github",
            Domain::Docker => "docker",
            Domain::Pg => "pg",
            Domain::Disk => "disk",
        }
    }
}

/// One discovery mark: when it lands, whether it fails.
#[derive(Debug, Clone)]
pub(crate) struct Discovery {
    pub(crate) domain: Domain,
    pub(crate) at_ms: i64,
    pub(crate) fails: bool,
    pub(crate) done: bool,
    pub(crate) failed: bool,
}

impl Discovery {
    pub(crate) fn at(domain: Domain, at_ms: i64) -> Self {
        Self {
            domain,
            at_ms,
            fails: false,
            done: false,
            failed: false,
        }
    }

    pub(crate) fn failing(domain: Domain, at_ms: i64) -> Self {
        Self {
            fails: true,
            ..Self::at(domain, at_ms)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Msg {
    /// A capability domain finished discovering.
    Discovered(Domain),
    /// Discovery of this domain failed (hard-cases honesty).
    DiscoveryFailed(Domain),
}

pub(crate) struct World {
    /// Monotonic simulation commit revision; never rendered or driven by ticks.
    pub(crate) effect_revision: u64,
    pub(crate) scenario: Scenario,
    pub(crate) clock: Clock,
    pub(crate) host: Host,
    /// Where holla was launched — the primary context object.
    pub(crate) cwd: String,
    pub(crate) discovery: Vec<Discovery>,
    // Fixture slots; `None` = discovery ran and found nothing here.
    pub(crate) mise: Option<MiseState>,
    pub(crate) git: Option<GitRepo>,
    pub(crate) ssh: Vec<SshHost>,
    pub(crate) github: Option<GhState>,
    pub(crate) docker: Option<DockerState>,
    pub(crate) pg: Option<Vec<PgSession>>,
    pub(crate) disk: Option<DiskState>,
    pub(crate) debian: Option<DebianState>,
    pub(crate) memory: Memory,
    pub(crate) activities: Vec<Activity>,
}

impl World {
    pub(crate) fn new(scenario: Scenario, host: Host, cwd: &str) -> Self {
        Self {
            effect_revision: 0,
            scenario,
            clock: Clock::new(),
            host,
            cwd: cwd.to_owned(),
            discovery: vec![],
            mise: None,
            git: None,
            ssh: vec![],
            github: None,
            docker: None,
            pg: None,
            disk: None,
            debian: None,
            memory: Memory::default(),
            activities: vec![],
        }
    }

    pub(crate) fn now_ms(&self) -> i64 {
        self.clock.now_ms
    }

    /// Default progressive schedule; scenarios may override marks.
    pub(crate) fn default_schedule() -> Vec<Discovery> {
        vec![
            Discovery::at(Domain::Mise, 600),
            Discovery::at(Domain::Git, 900),
            Discovery::at(Domain::Ssh, 1_200),
            Discovery::at(Domain::Github, 1_600),
            Discovery::at(Domain::Docker, 2_400),
            Discovery::at(Domain::Pg, 2_800),
            Discovery::at(Domain::Disk, 3_200),
        ]
    }

    /// Any domain still pending (failures settle: they are known, not pending).
    pub(crate) fn discovering(&self) -> bool {
        self.discovery.iter().any(|d| !d.done && !d.failed)
    }

    #[allow(dead_code)] // fixture tests + P2
    pub(crate) fn discovered(&self, domain: Domain) -> bool {
        self.discovery.iter().any(|d| d.domain == domain && d.done)
    }

    #[allow(dead_code)] // fixture tests + P2
    pub(crate) fn discovery_failed(&self, domain: Domain) -> bool {
        self.discovery
            .iter()
            .any(|d| d.domain == domain && d.failed)
    }

    /// Advance virtual time and emit due events. No-op while paused.
    pub(crate) fn tick(&mut self, interval_ms: i64) -> Vec<Msg> {
        if !self.clock.running {
            return vec![];
        }
        self.clock.advance(interval_ms);
        self.step()
    }

    /// Fast-forward to a fixture tick for `--motion paused --frame N`.
    ///
    /// Historical frames round upward in 80 ms increments from the current
    /// instant. Discovery only compares deadlines, so crossing every tick adds
    /// no information. Advance once to the same rounded instant; clamp values
    /// outside the signed fixture-clock range rather than wrap or loop.
    pub(crate) fn seek(&mut self, frame_ms: u64) {
        let target = i64::try_from(frame_ms).unwrap_or(i64::MAX);
        if target <= self.clock.now_ms {
            return;
        }
        let distance = target.saturating_sub(self.clock.now_ms);
        let ticks = distance / 80 + i64::from(distance % 80 != 0);
        self.clock.now_ms = self.clock.now_ms.saturating_add(ticks.saturating_mul(80));
        self.step();
    }

    /// Trust one exact mise task file; affected tasks re-resolve as Ready.
    pub(crate) fn trust_mise_file(&mut self, path: &str) -> bool {
        let Some(mise) = &mut self.mise else {
            return false;
        };
        let mut hit = false;
        for t in &mut mise.tasks {
            if t.defined_in == path && t.trust == crate::domain::mise::Trust::Untrusted {
                t.trust = crate::domain::mise::Trust::Trusted;
                hit = true;
            }
        }
        hit
    }

    /// Shared state evolution for tick and seek: land due discoveries.
    fn step(&mut self) -> Vec<Msg> {
        let now = self.clock.now_ms;
        let mut msgs = vec![];
        for d in &mut self.discovery {
            if d.done || d.failed || now < d.at_ms {
                continue;
            }
            if d.fails {
                d.failed = true;
                msgs.push(Msg::DiscoveryFailed(d.domain));
            } else {
                d.done = true;
                msgs.push(Msg::Discovered(d.domain));
            }
        }
        msgs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::host::Environment;

    #[test]
    fn discovery_is_progressive_and_failures_settle() {
        let mut w = World::new(Scenario::FirstUse, Host::local("devbox"), "~/scratch/empty");
        w.discovery = vec![
            Discovery::at(Domain::Git, 100),
            Discovery::failing(Domain::Docker, 200),
        ];
        assert!(w.discovering());
        w.clock.running = true;
        let msgs = w.tick(150);
        assert_eq!(msgs, vec![Msg::Discovered(Domain::Git)]);
        assert!(w.discovering());
        let msgs = w.tick(100);
        assert_eq!(msgs, vec![Msg::DiscoveryFailed(Domain::Docker)]);
        assert!(!w.discovering());
        assert!(w.discovery_failed(Domain::Docker));
        assert!(w.discovered(Domain::Git));
        // failures do not refire
        assert!(w.tick(100).is_empty());
        let _ = Environment::Local;
    }
    #[test]
    fn seek_preserves_reference_quantization_and_pause() {
        let mut w = World::new(Scenario::FirstUse, Host::local("devbox"), "~/scratch/empty");
        w.clock.running = false;
        w.seek(100);
        assert_eq!(w.now_ms(), 160);
        assert!(!w.clock.running);
        w.seek(80);
        assert_eq!(w.now_ms(), 160, "seek never moves backwards");
        w.clock.now_ms = 33;
        w.seek(100);
        assert_eq!(
            w.now_ms(),
            113,
            "quantization starts at the current instant"
        );
    }

    #[test]
    fn seek_maximum_frame_is_bounded_and_settles_discovery() {
        let mut w = World::new(Scenario::FirstUse, Host::local("devbox"), "~/scratch/empty");
        w.discovery = World::default_schedule();
        w.seek(u64::MAX);
        assert_eq!(w.now_ms(), i64::MAX);
        assert!(!w.discovering());
        assert!(w.tick(80).is_empty());
        assert_eq!(w.now_ms(), i64::MAX);
    }
}
