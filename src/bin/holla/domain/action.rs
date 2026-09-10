//! The result model: one type for every row the root can show (a direct
//! action, a resource with a primary action, a recommendation, a domain flow
//! or a specialist handoff), with scope, risk, confirmation, reasons and the
//! exact command it stands for. Capability, action and recommendation stay
//! distinct: a capability is a domain being present, an action is an `Item`,
//! a recommendation is an `Item` with a live-state reason.

use crate::domain::context::ScopeTag;
use crate::domain::effect::Effect;
use crate::domain::exec::Command;
use crate::sim::world::BatchMode;

/// The type word shown in the row's `type` column (D-6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    Mise,
    Git,
    Docker,
    System,
    Disk,
    Files,
    Postgres,
    Rust,
    Ssh,
    Cleanup,
    Plan,
    Activity,
    Explore,
    Personal,
    Node,
    Just,
    Make,
    Taskfile,
    Brew,
    Gradle,
    Idea,
    Upgrade,
}

impl Kind {
    pub fn label(self) -> &'static str {
        match self {
            Kind::Mise => "task",
            Kind::Git => "git",
            Kind::Docker => "docker",
            Kind::System => "system",
            Kind::Disk => "disk",
            Kind::Files => "file",
            Kind::Postgres => "postgres",
            Kind::Rust => "cargo",
            Kind::Ssh => "ssh",
            Kind::Cleanup => "cleanup",
            Kind::Plan => "plan",
            Kind::Activity => "activity",
            Kind::Explore => "explore",
            Kind::Personal => "custom",
            Kind::Node => "script",
            Kind::Just => "recipe",
            Kind::Make => "target",
            Kind::Taskfile => "task",
            Kind::Brew => "service",
            Kind::Gradle => "gradle",
            Kind::Idea => "idea",
            Kind::Upgrade => "upgrade",
        }
    }
}

/// What a row represents (CONCEPT §6.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultType {
    Action,
    Resource,
    Recommendation,
    Flow,
    Handoff,
}

/// Risk class (CONCEPT §10). Risk changes treatment, never availability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Risk {
    ReadOnly,
    Mutating,
    Destructive,
    Privileged,
}

impl Risk {
    pub fn label(self) -> &'static str {
        match self {
            Risk::ReadOnly => "read-only",
            Risk::Mutating => "mutating",
            Risk::Destructive => "destructive",
            Risk::Privileged => "privileged",
        }
    }
}

/// Confirmation level (CONCEPT §10).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Confirmation {
    /// Runs on Enter.
    None,
    /// One facts review before running.
    One,
    /// Review, then a typed target-bound phrase. `broad` adds the
    /// `I UNDERSTAND:` prefix.
    TwoGate { phrase: String, broad: bool },
    /// The defining configuration is not trusted yet: review and trust it
    /// first, then re-resolve.
    Trust { config: String },
}

impl Confirmation {
    pub fn label(&self) -> String {
        match self {
            Confirmation::None => "runs directly".into(),
            Confirmation::One => "one confirmation".into(),
            Confirmation::TwoGate { broad, .. } => {
                if *broad {
                    "two gates · typed phrase with I UNDERSTAND".into()
                } else {
                    "two gates · typed phrase".into()
                }
            }
            Confirmation::Trust { config } => format!("trust required · {config}"),
        }
    }

    /// The exact phrase the second gate requires.
    pub fn phrase(&self) -> Option<String> {
        match self {
            Confirmation::TwoGate { phrase, broad } => Some(if *broad {
                format!("I UNDERSTAND: {phrase}")
            } else {
                phrase.clone()
            }),
            _ => None,
        }
    }
}

/// Is the information behind a row live, cached, partial or missing
/// (CONCEPT §5.5 #6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Freshness {
    Live { age_ms: i64 },
    Cached { age_ms: i64 },
    Partial { loaded: usize, total: usize },
    Loading,
    Unavailable(String),
}

impl Freshness {
    pub fn label(&self) -> String {
        let age = |ms: i64| {
            let s = ms / 1000;
            if s < 60 {
                format!("{s} s ago")
            } else if s < 3600 {
                format!("{} min ago", s / 60)
            } else {
                format!("{} h ago", s / 3600)
            }
        };
        match self {
            Freshness::Live { age_ms } => format!("live · {}", age(*age_ms)),
            Freshness::Cached { age_ms } => format!("cached · {}", age(*age_ms)),
            Freshness::Partial { loaded, total } => {
                format!("partial · {loaded} of {total} sources")
            }
            Freshness::Loading => "still discovering…".into(),
            Freshness::Unavailable(r) => format!("unavailable · {r}"),
        }
    }
}

/// The strongest signal behind a recommendation (CONCEPT §9 ranking order).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Signal {
    /// Stable default order only.
    Default,
    /// Frequency or recency anywhere.
    GlobalUse,
    /// Frequency or recency in this path or project.
    LocalUse,
    /// Context matched (a manifest here, a service here).
    Context,
    /// Live state demands it (behind, dirty, unhealthy, full).
    Urgency,
    /// The user pinned it here.
    Pin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reason {
    pub signal: Signal,
    pub text: String,
}

impl Reason {
    pub fn new(signal: Signal, text: impl Into<String>) -> Self {
        Self {
            signal,
            text: text.into(),
        }
    }
}

/// A structured argument (CONCEPT §6.7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgSpec {
    pub name: String,
    pub expected: String,
    pub default: String,
    pub options: Vec<String>,
    pub required: bool,
    pub secret: bool,
    pub help: String,
}

impl ArgSpec {
    pub fn text(name: &str, expected: &str, default: &str) -> Self {
        Self {
            name: name.into(),
            expected: expected.into(),
            default: default.into(),
            options: vec![],
            required: false,
            secret: false,
            help: String::new(),
        }
    }
    pub fn choice(name: &str, options: &[&str], default: usize) -> Self {
        Self {
            name: name.into(),
            expected: "one of".into(),
            default: options
                .get(default)
                .map(|s| (*s).to_owned())
                .unwrap_or_default(),
            options: options.iter().map(|s| (*s).to_owned()).collect(),
            required: true,
            secret: false,
            help: String::new(),
        }
    }
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }
    pub fn help(mut self, h: &str) -> Self {
        self.help = h.into();
        self
    }
}

/// What Enter does once every gate has passed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Launch {
    /// Start an activity: the item's `exec` runs; `script` names a fixture
    /// script when one exists, else the outcome table decides.
    Activity { script: Option<String> },
    /// Start several activities as one batch.
    Batch,
    /// Open the Disk flow for a path.
    Disk { path: String },
    /// Open the Files page (browser) at a path.
    Files { path: String },
    /// Open the home file search.
    Find,
    /// Open the Cleanup review, optionally filtered to one category.
    Cleanup { category: Option<String> },
    /// Open the macOS Top files scope.
    TopFiles,
    /// Copy a value to the clipboard (a path).
    Copy { value: String },
    /// Open a configuration diagnostics page.
    Config { path: String },
    /// Open a domain page: the finder filtered to one Explore group.
    Group(&'static str),
    /// Open a plan for review.
    Plan { plan: String },
    /// Hand off to a specialist tool as a persistent activity.
    Handoff { tool: String },
    /// Switch the root scope.
    Scope(crate::domain::context::Scope),
    /// Show a snapshot dialog (system resources, git status).
    Snapshot { snapshot: String },
    /// Open an existing activity tab.
    OpenActivity { id: String },
    /// Insert into the shell and leave (simulated as a status line).
    Insert,
}

/// One row the root can show.
#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    /// Stable identity for direct invocation and memory (`git.pull`).
    pub id: String,
    pub label: String,
    pub kind: Kind,
    pub rtype: ResultType,
    pub scope: ScopeTag,
    pub risk: Risk,
    pub confirmation: Confirmation,
    /// Plain-language sentence: what will happen.
    pub summary: String,
    /// The exact command sequence, one per line.
    pub commands: Vec<String>,
    /// What will change.
    pub effects: Vec<String>,
    /// Reasons this row is relevant, strongest first.
    pub reasons: Vec<Reason>,
    /// Extra words the search matches (intent phrases, resource names).
    pub keywords: Vec<String>,
    /// Exact aliases that beat ranking.
    pub aliases: Vec<String>,
    /// Aliases that apply only on the item's own domain page (`u` for Usage
    /// inside Disk).
    pub page_aliases: Vec<String>,
    pub freshness: Freshness,
    pub args: Vec<ArgSpec>,
    /// Alternative operations reachable from the alternatives menu.
    pub alternatives: Vec<Alternative>,
    pub launch: Launch,
    /// Which Explore group the row belongs to (`Git`, `Disk`, …).
    pub group: &'static str,
    /// Counts from usage memory, filled by the catalogue.
    pub used_here: u32,
    pub used_anywhere: u32,
    /// Fixture instant of the last use here, if any.
    pub last_used_secs: Option<i64>,
    pub pinned: bool,
    pub hidden: bool,
    /// The preferred specialist tool, when the user chose one.
    pub preferred_tool: Option<String>,
    /// The exact executed specification; `commands` is its display form.
    pub exec: Vec<Command>,
    /// What a successful run changes in the world.
    pub effect: Option<Effect>,
    /// Members of a batch action: name, spec, effect, scope.
    pub batch: Vec<(String, Vec<Command>, Option<Effect>, ScopeTag)>,
    pub batch_mode: BatchMode,
    /// Where the definition came from: provider, file and entry.
    pub provenance: String,
    /// Trust binding for a contributed definition: (path, digest).
    pub trust_key: Option<(String, String)>,
    /// Bounded listing note (`30 of 42 scripts`).
    pub cap_note: Option<String>,
    /// Ranking score contributions filled by the catalogue.
    pub frecency: f64,
}

/// An alternative operation on the same resource (CONCEPT §6.5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alternative {
    pub label: String,
    /// Item id the alternative maps to.
    pub item: String,
}

impl Item {
    #[allow(clippy::too_many_arguments)]
    pub fn new(id: &str, label: &str, kind: Kind, rtype: ResultType, scope: ScopeTag) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind,
            rtype,
            scope,
            risk: Risk::ReadOnly,
            confirmation: Confirmation::None,
            summary: String::new(),
            commands: vec![],
            effects: vec![],
            reasons: vec![],
            keywords: vec![],
            aliases: vec![],
            page_aliases: vec![],
            freshness: Freshness::Live { age_ms: 0 },
            args: vec![],
            alternatives: vec![],
            launch: Launch::Insert,
            group: "",
            used_here: 0,
            used_anywhere: 0,
            last_used_secs: None,
            pinned: false,
            hidden: false,
            preferred_tool: None,
            exec: vec![],
            effect: None,
            batch: vec![],
            batch_mode: BatchMode::Parallel,
            provenance: "built-in".into(),
            trust_key: None,
            cap_note: None,
            frecency: 0.0,
        }
    }
    /// The typed specification; the display commands derive from it.
    pub fn exec(mut self, cmds: Vec<Command>) -> Self {
        self.commands = crate::domain::exec::displays(&cmds);
        self.exec = cmds;
        self
    }
    pub fn effect(mut self, e: Effect) -> Self {
        self.effect = Some(e);
        self
    }
    pub fn provenance(mut self, p: &str) -> Self {
        self.provenance = p.into();
        self
    }
    pub fn batch(
        mut self,
        mode: BatchMode,
        members: Vec<(String, Vec<Command>, Option<Effect>, ScopeTag)>,
    ) -> Self {
        self.commands = members
            .iter()
            .flat_map(|(_, c, _, _)| crate::domain::exec::displays(c))
            .collect();
        self.batch = members;
        self.batch_mode = mode;
        self.launch = Launch::Batch;
        self
    }
    /// Every command this row would execute, batch members included.
    pub fn all_exec(&self) -> Vec<Command> {
        if self.batch.is_empty() {
            self.exec.clone()
        } else {
            self.batch
                .iter()
                .flat_map(|(_, c, _, _)| c.clone())
                .collect()
        }
    }
    pub fn risk(mut self, r: Risk) -> Self {
        self.risk = r;
        self
    }
    pub fn confirm(mut self, c: Confirmation) -> Self {
        self.confirmation = c;
        self
    }
    pub fn summary(mut self, s: &str) -> Self {
        self.summary = s.into();
        self
    }
    pub fn effects(mut self, e: &[&str]) -> Self {
        self.effects = e.iter().map(|s| (*s).to_owned()).collect();
        self
    }
    pub fn reason(mut self, signal: Signal, text: &str) -> Self {
        self.reasons.push(Reason::new(signal, text));
        self
    }
    pub fn keywords(mut self, k: &[&str]) -> Self {
        self.keywords = k.iter().map(|s| (*s).to_owned()).collect();
        self
    }
    pub fn aliases(mut self, a: &[&str]) -> Self {
        self.aliases = a.iter().map(|s| (*s).to_owned()).collect();
        self
    }
    pub fn page_aliases(mut self, a: &[&str]) -> Self {
        self.page_aliases = a.iter().map(|s| (*s).to_owned()).collect();
        self
    }
    pub fn freshness(mut self, f: Freshness) -> Self {
        self.freshness = f;
        self
    }
    pub fn args(mut self, a: Vec<ArgSpec>) -> Self {
        self.args = a;
        self
    }
    pub fn alt(mut self, label: &str, item: &str) -> Self {
        self.alternatives.push(Alternative {
            label: label.into(),
            item: item.into(),
        });
        self
    }
    pub fn launch(mut self, l: Launch) -> Self {
        self.launch = l;
        self
    }
    pub fn group(mut self, g: &'static str) -> Self {
        self.group = g;
        self
    }

    /// The strongest reason, for the row.
    pub fn top_reason(&self) -> Option<&Reason> {
        self.reasons.iter().max_by_key(|r| r.signal)
    }

    /// The strongest signal, or `Default`.
    pub fn signal(&self) -> Signal {
        self.top_reason()
            .map(|r| r.signal)
            .unwrap_or(Signal::Default)
    }

    /// True when live state makes this a recommendation rather than a
    /// merely available action.
    pub fn is_recommended(&self) -> bool {
        self.signal() >= Signal::LocalUse
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phrases_carry_the_broad_prefix() {
        let c = Confirmation::TwoGate {
            phrase: "REMOVE ALL DOCKER DATA ON devbox".into(),
            broad: true,
        };
        assert_eq!(
            c.phrase().as_deref(),
            Some("I UNDERSTAND: REMOVE ALL DOCKER DATA ON devbox")
        );
        let c = Confirmation::TwoGate {
            phrase: "DELETE EVERYTHING IN /work/scratch".into(),
            broad: false,
        };
        assert_eq!(
            c.phrase().as_deref(),
            Some("DELETE EVERYTHING IN /work/scratch")
        );
        assert_eq!(Confirmation::None.phrase(), None);
    }

    #[test]
    fn strongest_reason_wins_and_marks_a_recommendation() {
        let i = Item::new(
            "x",
            "X",
            Kind::Git,
            ResultType::Action,
            ScopeTag::here("/a"),
        )
        .reason(Signal::LocalUse, "used here")
        .reason(Signal::Urgency, "branch is 3 commits behind");
        assert_eq!(i.top_reason().unwrap().text, "branch is 3 commits behind");
        assert!(i.is_recommended());
        let j = Item::new(
            "y",
            "Y",
            Kind::Git,
            ResultType::Action,
            ScopeTag::here("/a"),
        )
        .reason(Signal::Default, "available");
        assert!(!j.is_recommended());
    }

    #[test]
    fn freshness_labels_read_as_sentences() {
        assert_eq!(Freshness::Live { age_ms: 2_000 }.label(), "live · 2 s ago");
        assert_eq!(
            Freshness::Cached { age_ms: 200_000 }.label(),
            "cached · 3 min ago"
        );
        assert_eq!(
            Freshness::Partial {
                loaded: 4,
                total: 6
            }
            .label(),
            "partial · 4 of 6 sources"
        );
    }
}
