use crate::activity_supervisor::{
    ActivityExecutionId, ActivityExecutionStart, ActivityStopProfile, ActivitySupervisor,
    ActivitySupervisorError,
};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Domain {
    Mise,
    Git,
    Ssh,
    Github,
    Docker,
    Pg,
    Disk,
}

impl Domain {
    pub const ALL: [Domain; 7] = [
        Domain::Mise,
        Domain::Git,
        Domain::Ssh,
        Domain::Github,
        Domain::Docker,
        Domain::Pg,
        Domain::Disk,
    ];

    pub fn label(self) -> &'static str {
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

#[derive(Debug, Clone)]
pub struct Discovery {
    pub domain: Domain,
    pub at_ms: i64,
    pub fails: bool,
    pub done: bool,
    pub failed: bool,
}

impl Discovery {
    pub fn at(domain: Domain, at_ms: i64) -> Self {
        Self {
            domain,
            at_ms,
            fails: false,
            done: false,
            failed: false,
        }
    }

    pub fn failing(domain: Domain, at_ms: i64) -> Self {
        Self {
            fails: true,
            ..Self::at(domain, at_ms)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Msg {
    Discovered(Domain),
    DiscoveryFailed(Domain),
}

pub struct World {
    pub effect_revision: u64,
    pub scenario: Scenario,
    pub clock: Clock,
    pub activity_supervisor: ActivitySupervisor,
    pub host: Host,
    pub cwd: String,
    pub discovery: Vec<Discovery>,

    pub mise: Option<MiseState>,
    pub git: Option<GitRepo>,
    pub ssh: Vec<SshHost>,
    pub github: Option<GhState>,
    pub docker: Option<DockerState>,
    pub pg: Option<Vec<PgSession>>,
    pub disk: Option<DiskState>,
    pub debian: Option<DebianState>,
    pub memory: Memory,
    pub activities: Vec<Activity>,
    scenario_activity_execution_ids: Vec<ActivityExecutionId>,
}

impl World {
    pub fn new(scenario: Scenario, host: Host, cwd: &str) -> Self {
        Self {
            effect_revision: 0,
            scenario,
            clock: Clock::new(),
            activity_supervisor: ActivitySupervisor::new(),
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
            scenario_activity_execution_ids: Vec::new(),
        }
    }

    /// Registers and publishes one Activity during deterministic scenario setup.
    ///
    /// Rejects an ID already present in `activities` before changing the supervisor. The exact
    /// returned identity is retained for App fixture adoption; rendering must not reconstruct it.
    ///
    /// # Errors
    /// Returns [`ActivitySupervisorError::DuplicateActivityId`] when the display ID already exists
    /// in the World or supervisor, or [`ActivitySupervisorError::GenerationExhausted`] when no
    /// supervisor identity remains.
    pub fn register_scenario_activity(
        &mut self,
        activity: Activity,
        start: ActivityExecutionStart,
        profile: ActivityStopProfile,
    ) -> Result<ActivityExecutionId, ActivitySupervisorError> {
        if self
            .activities
            .iter()
            .any(|existing| existing.id == activity.id)
        {
            return Err(ActivitySupervisorError::DuplicateActivityId {
                activity_id: activity.id,
            });
        }

        let execution_id = self
            .activity_supervisor
            .register(&activity, start, profile)?;
        self.activities.push(activity);
        self.scenario_activity_execution_ids.push(execution_id);
        Ok(execution_id)
    }

    /// Returns exact supervisor identities captured during scenario setup.
    pub fn scenario_activity_execution_ids(&self) -> &[ActivityExecutionId] {
        &self.scenario_activity_execution_ids
    }

    pub fn now_ms(&self) -> i64 {
        self.clock.now_ms
    }

    pub fn default_schedule() -> Vec<Discovery> {
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

    pub fn discovering(&self) -> bool {
        self.discovery.iter().any(|d| !d.done && !d.failed)
    }

    /// Returns whether every scheduled discovery has reached a terminal state.
    ///
    /// An empty discovery list is settled. This reports discovery state only;
    /// it does not advance the clock or run pending discovery work.
    pub fn settled(&self) -> bool {
        self.discovery.iter().all(|d| d.done || d.failed)
    }

    pub fn discovered(&self, domain: Domain) -> bool {
        self.discovery.iter().any(|d| d.domain == domain && d.done)
    }

    pub fn discovery_failed(&self, domain: Domain) -> bool {
        self.discovery
            .iter()
            .any(|d| d.domain == domain && d.failed)
    }

    pub fn tick(&mut self, interval_ms: i64) -> Vec<Msg> {
        if !self.clock.running {
            return vec![];
        }
        self.clock.advance(interval_ms);
        self.step()
    }

    pub fn seek(&mut self, frame_ms: u64) {
        let target = i64::try_from(frame_ms).unwrap_or(i64::MAX);
        if target <= self.clock.now_ms {
            return;
        }
        let distance = target.saturating_sub(self.clock.now_ms);
        let ticks = (distance / 80).saturating_add(i64::from(distance % 80 != 0));
        self.clock.now_ms = self.clock.now_ms.saturating_add(ticks.saturating_mul(80));
        self.step();
    }

    pub fn trust_mise_file(&mut self, path: &str) -> bool {
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

    fn step(&mut self) -> Vec<Msg> {
        let now = self.clock.now_ms;
        self.activity_supervisor.advance_to(now);
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
    use crate::activity_supervisor::{
        ActivityExecutionPhase, ActivityExecutionStart, ActivityStopMilestone, ActivityStopPhase,
        ActivityStopProfile,
    };
    use crate::domain::activity::{Activity, ActivityState};
    use crate::domain::host::Environment;
    use std::error::Error;

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

        assert!(w.tick(100).is_empty());
        let _ = Environment::Local;
    }

    #[test]
    fn scenario_registration_rejects_existing_activity_before_supervisor_mutation() {
        let mut world = World::new(
            Scenario::ParityActivityStop,
            Host::local("devbox"),
            "~/work/activity-stop",
        );
        let existing = Activity::new(
            7,
            "static row",
            "~/work/activity-stop",
            ActivityState::Running,
            0,
        );
        world.activities.push(existing.clone());

        let result = world.register_scenario_activity(
            existing.clone(),
            ActivityExecutionStart::Running,
            ActivityStopProfile::Responsive,
        );

        assert_eq!(
            result,
            Err(ActivitySupervisorError::DuplicateActivityId { activity_id: 7 })
        );
        assert_eq!(world.activities, vec![existing]);
        assert!(world.scenario_activity_execution_ids().is_empty());
        assert_eq!(world.activity_supervisor.current_execution_id(7), None);
    }

    #[test]
    fn settled_is_true_for_an_empty_discovery_list() {
        let w = World::new(Scenario::FirstUse, Host::local("devbox"), "~/scratch/empty");
        assert!(w.settled());
    }

    #[test]
    fn settled_requires_each_discovery_to_finish_or_fail() {
        let mut w = World::new(Scenario::FirstUse, Host::local("devbox"), "~/scratch/empty");
        w.discovery = vec![
            Discovery {
                done: true,
                ..Discovery::at(Domain::Git, 100)
            },
            Discovery {
                failed: true,
                ..Discovery::failing(Domain::Docker, 200)
            },
            Discovery::at(Domain::Ssh, 300),
        ];

        assert!(!w.settled());

        w.clock.now_ms = 1_000;
        assert!(!w.settled(), "time alone does not finish a discovery");
        w.discovery[2].done = true;
        assert!(w.settled());
    }

    #[test]
    fn settled_is_true_when_all_discoveries_succeed() {
        let mut w = World::new(Scenario::FirstUse, Host::local("devbox"), "~/scratch/empty");
        w.discovery = vec![
            Discovery {
                done: true,
                ..Discovery::at(Domain::Git, 100)
            },
            Discovery {
                done: true,
                ..Discovery::at(Domain::Docker, 200)
            },
        ];

        w.clock.now_ms = 0;
        assert!(w.settled(), "settled state does not depend on clock time");
    }

    #[test]
    fn settled_is_true_when_all_discoveries_fail() {
        let mut w = World::new(Scenario::FirstUse, Host::local("devbox"), "~/scratch/empty");
        w.discovery = vec![
            Discovery {
                failed: true,
                ..Discovery::failing(Domain::Git, 100)
            },
            Discovery {
                failed: true,
                ..Discovery::failing(Domain::Docker, 200)
            },
        ];

        assert!(w.settled());
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

    #[test]
    fn tick_advances_activity_stop_reaping_at_the_deadline() -> Result<(), Box<dyn Error>> {
        let mut world = World::new(Scenario::FirstUse, Host::local("devbox"), "~/scratch/empty");
        let activity = Activity::new(1, "worker", "~/work", ActivityState::Running, 0);
        let id = world.activity_supervisor.register(
            &activity,
            ActivityExecutionStart::Running,
            ActivityStopProfile::Responsive,
        )?;
        let now_ms = world.now_ms();
        world
            .activity_supervisor
            .admit_stop(id, &activity, now_ms)?
            .commit();

        assert!(world.tick(239).is_empty());
        assert_eq!(
            world
                .activity_supervisor
                .execution(id)
                .map(|execution| execution.phase()),
            Some(ActivityExecutionPhase::Cancelling)
        );
        assert!(world.tick(1).is_empty());
        assert_eq!(
            world
                .activity_supervisor
                .execution(id)
                .map(|execution| execution.phase()),
            Some(ActivityExecutionPhase::Stopped)
        );
        assert_eq!(world.effect_revision, 0);
        Ok(())
    }

    #[test]
    fn seek_advances_activity_stop_milestones_at_scheduled_times() -> Result<(), Box<dyn Error>> {
        let mut world = World::new(Scenario::FirstUse, Host::local("devbox"), "~/scratch/empty");
        let activity = Activity::new(2, "detached", "~/work", ActivityState::Running, 0);
        let id = world.activity_supervisor.register(
            &activity,
            ActivityExecutionStart::Detached,
            ActivityStopProfile::Resistant,
        )?;
        let now_ms = world.now_ms();
        world
            .activity_supervisor
            .admit_stop(id, &activity, now_ms)?
            .commit();

        world.seek(1_000);
        let execution = world
            .activity_supervisor
            .execution(id)
            .ok_or("execution missing")?;
        assert_eq!(execution.phase(), ActivityExecutionPhase::Stopped);
        assert_eq!(execution.stop_phase(), ActivityStopPhase::Reaped);
        assert_eq!(
            execution.milestones(),
            &[
                ActivityStopMilestone::TermSent { at_ms: 0 },
                ActivityStopMilestone::KillSent { at_ms: 800 },
                ActivityStopMilestone::Reaped { at_ms: 960 },
            ]
        );
        assert_eq!(world.effect_revision, 0);
        Ok(())
    }
}
