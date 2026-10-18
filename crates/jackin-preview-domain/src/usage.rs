#![doc = r" Usage projection: quota windows, freshness, status, and the honest"]
#![doc = r" overall aggregation across every account and provider."]

use std::collections::BTreeMap;

use super::account::{Account, AccountId, Lifecycle};
use super::agent::{Provider, UsageSurface};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[doc = r" Freshness state of an account's usage data."]
pub enum Freshness {
    #[doc = r" Usage data is current."]
    Current,
    #[doc = r" Usage data is retained from an earlier successful refresh."]
    Stale,
    #[doc = r" A refresh is currently in progress."]
    Refreshing,
    #[doc = r" The latest refresh failed."]
    Failed,
}

impl Freshness {
    #[doc = r" Return the stable display label for this freshness state."]
    pub fn label(self) -> &'static str {
        match self {
            Freshness::Current => "current",
            Freshness::Stale => "stale",
            Freshness::Refreshing => "refreshing",
            Freshness::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[doc = r" Semantic availability state for a quota window."]
pub enum QuotaStatus {
    #[doc = r" Usage is below the warning threshold."]
    Available,
    #[doc = r" The provider has not started tracking this window."]
    NotStarted,
    #[doc = r" Usage has reached the warning threshold."]
    Warning,
    #[doc = r" Usage has reached or exceeded the limit."]
    Exhausted,
    #[doc = r" The provider does not expose this quota."]
    Unsupported,
    #[doc = r" The quota cannot currently be retrieved."]
    Unavailable,
    #[doc = r" Retrieving the quota produced an error."]
    Error,
}

impl QuotaStatus {
    #[doc = r" Return the stable display label for this status."]
    pub fn label(self) -> &'static str {
        match self {
            QuotaStatus::Available => "available",
            QuotaStatus::NotStarted => "not started",
            QuotaStatus::Warning => "warning",
            QuotaStatus::Exhausted => "exhausted",
            QuotaStatus::Unsupported => "unsupported",
            QuotaStatus::Unavailable => "unavailable",
            QuotaStatus::Error => "error",
        }
    }

    #[doc = r" Derive from a used percentage (warning at 75 %, exhausted at 100 %)."]
    pub fn from_pct(used_pct: u8) -> Self {
        if used_pct >= 100 {
            QuotaStatus::Exhausted
        } else if used_pct >= 75 {
            QuotaStatus::Warning
        } else {
            QuotaStatus::Available
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[doc = r" Semantic category used to compare quota windows."]
pub enum WindowCategory {
    #[doc = r" Short-lived session quota."]
    Session,
    #[doc = r" Quota covering a longer period, such as a week or month."]
    LongRange,
    #[doc = r" Quota scoped to a model."]
    Model,
    #[doc = r" Quota that does not fit another category."]
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[doc = r" Unit used to display a quota window's usage."]
pub enum WindowUnit {
    #[doc = r" A percentage of the quota used."]
    Percent,
    #[doc = r" A token count."]
    Tokens,
    #[doc = r" A credit count."]
    Credits,
    #[doc = r" A U.S. dollar amount."]
    Usd,
}

impl WindowUnit {
    #[doc = r" Return the stable display label for this unit."]
    pub fn label(self) -> &'static str {
        match self {
            WindowUnit::Percent => "%",
            WindowUnit::Tokens => "tokens",
            WindowUnit::Credits => "credits",
            WindowUnit::Usd => "USD",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[doc = r" Usage measurements and display metadata for one quota window."]
pub struct QuotaWindow {
    #[doc = r" Stable per surface: `session`, `weekly`, `credits`…"]
    pub id: &'static str,
    #[doc = r" Human-readable window label."]
    pub label: String,
    #[doc = r" Semantic category used for aggregation."]
    pub category: WindowCategory,
    #[doc = r" Measurement unit for the usage values."]
    pub unit: WindowUnit,
    #[doc = r" Observed usage count, when the provider reports one."]
    pub used: Option<u64>,
    #[doc = r" Usage limit, when the provider reports one."]
    pub limit: Option<u64>,
    #[doc = r" Usage percentage, capped at 100% when constructed by [`Self::pct`]."]
    pub used_pct: Option<u8>,
    #[doc = r" Fixture timestamp in seconds when the window resets."]
    pub reset_secs: Option<i64>,
    #[doc = r" Optional human-readable spend summary."]
    pub spend_label: Option<String>,
    #[doc = r" Current semantic status of the window."]
    pub status: QuotaStatus,
    #[doc = r" Optional explanatory note shown without numeric usage."]
    pub note: Option<String>,
}

impl QuotaWindow {
    #[doc = r" Create a percentage-based window, clamping its usage to 100%."]
    pub fn pct(id: &'static str, label: &str, category: WindowCategory, used_pct: u8) -> Self {
        Self {
            id,
            label: label.to_owned(),
            category,
            unit: WindowUnit::Percent,
            used: None,
            limit: None,
            used_pct: Some(used_pct.min(100)),
            reset_secs: None,
            spend_label: None,
            status: QuotaStatus::from_pct(used_pct),
            note: None,
        }
    }

    #[doc = r" Create a count-based window from its used and limit values."]
    pub fn counted(
        id: &'static str,
        label: &str,
        category: WindowCategory,
        unit: WindowUnit,
        used: u64,
        limit: u64,
    ) -> Self {
        let pct = (used * 100).checked_div(limit).unwrap_or(0).min(100) as u8;
        Self {
            id,
            label: label.to_owned(),
            category,
            unit,
            used: Some(used),
            limit: Some(limit),
            used_pct: Some(pct),
            reset_secs: None,
            spend_label: None,
            status: QuotaStatus::from_pct(pct),
            note: None,
        }
    }

    #[doc = r" Create a window with no numeric meter and an explanatory note."]
    pub fn sentinel(id: &'static str, label: &str, status: QuotaStatus, note: &str) -> Self {
        Self {
            id,
            label: label.to_owned(),
            category: WindowCategory::Other,
            unit: WindowUnit::Percent,
            used: None,
            limit: None,
            used_pct: None,
            reset_secs: None,
            spend_label: None,
            status,
            note: Some(note.to_owned()),
        }
    }

    #[doc = r" Create a zero-use window marked as not started."]
    pub fn not_started(id: &'static str, label: &str, category: WindowCategory) -> Self {
        let mut w = Self::pct(id, label, category, 0);
        w.status = QuotaStatus::NotStarted;
        w
    }

    #[doc = r" Set the reset timestamp in fixture seconds."]
    pub fn reset(mut self, secs: i64) -> Self {
        self.reset_secs = Some(secs);
        self
    }

    #[doc = r" Attach a human-readable spend summary."]
    pub fn spend(mut self, label: &str) -> Self {
        self.spend_label = Some(label.to_owned());
        self
    }

    #[doc = r" Override the window's derived status."]
    pub fn status(mut self, s: QuotaStatus) -> Self {
        self.status = s;
        self
    }

    #[doc = r" Return the remaining percentage when usage is known."]
    pub fn remaining_pct(&self) -> Option<u8> {
        self.used_pct.map(|p| 100 - p)
    }

    #[doc = r" `62% used`, `1,240 / 5,000 credits`, `not started`."]
    pub fn value_label(&self) -> String {
        match (self.used, self.limit, self.used_pct, self.status) {
            (_, _, _, QuotaStatus::NotStarted) => "not started".into(),
            (Some(u), Some(l), _, _) if self.unit != WindowUnit::Percent => format!(
                "{} / {} {}",
                thousands(u as usize),
                thousands(l as usize),
                self.unit.label()
            ),
            (_, _, Some(p), _) => format!("{p}% used"),
            _ => self
                .note
                .clone()
                .unwrap_or_else(|| self.status.label().to_owned()),
        }
    }

    #[doc = r" Report whether the window should render a meter."]
    pub fn has_meter(&self) -> bool {
        self.used_pct.is_some()
            && !matches!(
                self.status,
                QuotaStatus::Unsupported | QuotaStatus::Unavailable | QuotaStatus::Error
            )
    }
}

fn thousands(value: usize) -> String {
    let digits = value.to_string();
    let mut out = String::with_capacity(digits.len().saturating_add(digits.len() / 3));
    for (index, ch) in digits.chars().enumerate() {
        if index > 0 && (digits.len().saturating_sub(index)).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[doc = r" Freshness phase and timestamps for one account's usage projection."]
pub struct FreshnessInfo {
    #[doc = r" Current freshness phase."]
    pub phase: Freshness,
    #[doc = r" Fixture timestamp of the most recent successful refresh."]
    pub last_good_secs: Option<i64>,
    #[doc = r" Fixture timestamp of the next retry, when scheduled."]
    pub retry_secs: Option<i64>,
}

impl FreshnessInfo {
    #[doc = r" Construct current freshness at the given fixture timestamp."]
    pub fn current(at: i64) -> Self {
        Self {
            phase: Freshness::Current,
            last_good_secs: Some(at),
            retry_secs: None,
        }
    }
    #[doc = r" Construct stale freshness with the last-good and retry timestamps."]
    pub fn stale(last_good: i64, retry: i64) -> Self {
        Self {
            phase: Freshness::Stale,
            last_good_secs: Some(last_good),
            retry_secs: Some(retry),
        }
    }
    #[doc = r" Construct refreshing freshness, optionally retaining a last-good timestamp."]
    pub fn refreshing(last_good: Option<i64>) -> Self {
        Self {
            phase: Freshness::Refreshing,
            last_good_secs: last_good,
            retry_secs: None,
        }
    }
    #[doc = r" Construct failed freshness with optional retained and retry timestamps."]
    pub fn failed(last_good: Option<i64>, retry: Option<i64>) -> Self {
        Self {
            phase: Freshness::Failed,
            last_good_secs: last_good,
            retry_secs: retry,
        }
    }
}

#[doc = r" Per-account usage: last-good windows retained across stale/failed."]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountUsage {
    #[doc = r" Freshness metadata for the account's usage data."]
    pub freshness: FreshnessInfo,
    #[doc = r" Quota windows reported for the account."]
    pub windows: Vec<QuotaWindow>,
}

impl AccountUsage {
    #[doc = r" Construct an account usage projection with no windows and failed freshness."]
    pub fn none() -> Self {
        Self {
            freshness: FreshnessInfo::failed(None, None),
            windows: vec![],
        }
    }

    #[doc = r" Return the highest-severity status among the account's windows."]
    pub fn worst_status(&self) -> Option<QuotaStatus> {
        self.windows
            .iter()
            .map(|w| w.status)
            .max_by_key(|s| match s {
                QuotaStatus::Exhausted => 6,
                QuotaStatus::Error => 5,
                QuotaStatus::Warning => 4,
                QuotaStatus::Unavailable => 3,
                QuotaStatus::Unsupported => 2,
                QuotaStatus::Available => 1,
                QuotaStatus::NotStarted => 0,
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[doc = r" Overall health classification derived from account usage data."]
pub enum HealthWord {
    #[doc = r" No accounts are present."]
    Empty,
    #[doc = r" All enabled accounts are blocked or failed."]
    Blocked,
    #[doc = r" Usage data has a material failure, staleness, or exhaustion."]
    Degraded,
    #[doc = r" Usage data is usable but needs attention."]
    Attention,
    #[doc = r" No account-level issue was found."]
    Healthy,
}

impl HealthWord {
    #[doc = r" Return the stable display label for this health word."]
    pub fn label(self) -> &'static str {
        match self {
            HealthWord::Empty => "empty",
            HealthWord::Blocked => "blocked",
            HealthWord::Degraded => "degraded",
            HealthWord::Attention => "attention",
            HealthWord::Healthy => "healthy",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[doc = r" Counts used to summarize account and usage health."]
pub struct OverallCounts {
    #[doc = r" Total number of accounts."]
    pub accounts: usize,
    #[doc = r" Number of enabled accounts."]
    pub enabled: usize,
    #[doc = r" Number of disabled accounts."]
    pub disabled: usize,
    #[doc = r" Number of distinct providers."]
    pub providers: usize,
    #[doc = r" Number of warning windows."]
    pub warnings: usize,
    #[doc = r" Number of exhausted windows."]
    pub exhausted: usize,
    #[doc = r" Number of accounts with stale usage."]
    pub stale: usize,
    #[doc = r" Number of accounts currently refreshing usage."]
    pub refreshing: usize,
    #[doc = r" Number of accounts whose usage refresh failed."]
    pub failed: usize,
    #[doc = r" Number of accounts whose quota is not visible."]
    pub unsupported: usize,
    #[doc = r" Number of enabled accounts without a resolved identity."]
    pub unresolved_identity: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[doc = r" Aggregated quota window comparable across multiple accounts."]
pub struct ComparableRollup {
    #[doc = r" Provider surface represented by the rollup."]
    pub surface: UsageSurface,
    #[doc = r" Stable window identifier shared by contributing accounts."]
    pub window_id: &'static str,
    #[doc = r" Display label shared by contributing windows."]
    pub label: String,
    #[doc = r" Measurement unit shared by contributing windows."]
    pub unit: WindowUnit,
    #[doc = r" Number of contributing accounts."]
    pub accounts: usize,
    #[doc = r" Lowest remaining percentage among contributors."]
    pub min_remaining_pct: u8,
    #[doc = r" Highest remaining percentage among contributors."]
    pub max_remaining_pct: u8,
    #[doc = r" Summed (used, limit) for counted units."]
    pub summed: Option<(u64, u64)>,
    #[doc = r" Number of contributors represented by retained last-good data."]
    pub last_good_count: usize,
    #[doc = r" Number of unsupported windows on the same provider surface."]
    pub not_visible: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[doc = r" Explanation for why a provider surface cannot be aggregated."]
pub struct NotComparableNote {
    #[doc = r" Provider surface that could not be compared."]
    pub surface: UsageSurface,
    #[doc = r" Stable explanation for the lack of comparability."]
    pub reason: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[doc = r" Honest usage summary across all supplied accounts."]
pub struct OverallSummary {
    #[doc = r" Derived overall health classification."]
    pub health: HealthWord,
    #[doc = r" Counts of accounts and usage issues."]
    pub counts: OverallCounts,
    #[doc = r" Quota windows that can be compared across accounts."]
    pub comparable: Vec<ComparableRollup>,
    #[doc = r" Quota surfaces that could not be compared."]
    pub not_comparable: Vec<NotComparableNote>,
    #[doc = r" Account identifiers with stale usage data."]
    pub stale_sources: Vec<AccountId>,
    #[doc = r" Account identifiers whose usage refresh failed."]
    pub failed_sources: Vec<AccountId>,
}

impl OverallSummary {
    #[doc = r" Aggregate honestly: sum only identical (surface, window, unit,"]
    #[doc = r" category) counted windows; percentages roll up as a range."]
    pub fn compute(accounts: &[Account]) -> Self {
        let mut counts = OverallCounts {
            accounts: accounts.len(),
            ..Default::default()
        };
        let mut providers: BTreeMap<Provider, ()> = BTreeMap::new();
        let mut stale_sources = vec![];
        let mut failed_sources = vec![];

        let mut groups: BTreeMap<(UsageSurface, &'static str), Vec<(&QuotaWindow, bool)>> =
            BTreeMap::new();
        let mut not_visible: BTreeMap<UsageSurface, usize> = BTreeMap::new();
        let mut blocked = 0usize;
        for a in accounts {
            providers.insert(a.provider, ());
            if !a.enabled {
                counts.disabled += 1;
                continue;
            }
            counts.enabled += 1;
            if a.identity.subject.is_none() {
                counts.unresolved_identity += 1;
            }
            if matches!(
                a.lifecycle,
                Lifecycle::NeedsLogin | Lifecycle::NeedsSecret | Lifecycle::Error
            ) {
                blocked += 1;
            }
            match a.usage.freshness.phase {
                Freshness::Stale => {
                    counts.stale += 1;
                    stale_sources.push(a.id.clone());
                }
                Freshness::Refreshing => counts.refreshing += 1,
                Freshness::Failed => {
                    counts.failed += 1;
                    failed_sources.push(a.id.clone());
                }
                Freshness::Current => {}
            }
            let last_good = a.usage.freshness.phase != Freshness::Current;
            let all_unsupported = !a.usage.windows.is_empty()
                && a.usage
                    .windows
                    .iter()
                    .all(|w| w.status == QuotaStatus::Unsupported);
            if all_unsupported || a.lifecycle == Lifecycle::Unsupported {
                counts.unsupported += 1;
            }
            for w in &a.usage.windows {
                match w.status {
                    QuotaStatus::Warning => counts.warnings += 1,
                    QuotaStatus::Exhausted => counts.exhausted += 1,
                    QuotaStatus::Unsupported => {
                        *not_visible.entry(a.surface).or_default() += 1;
                        continue;
                    }
                    _ => {}
                }
                if w.used_pct.is_some() {
                    groups
                        .entry((a.surface, w.id))
                        .or_default()
                        .push((w, last_good));
                }
            }
        }
        counts.providers = providers.len();
        let mut comparable = vec![];
        let mut not_comparable = vec![];
        let mut seen_single: BTreeMap<UsageSurface, Vec<&'static str>> = BTreeMap::new();
        for ((surface, id), ws) in &groups {
            if ws.len() < 2 {
                seen_single.entry(*surface).or_default().push(id);
                continue;
            }
            let Some((first_window, _)) = ws.first() else {
                continue;
            };
            let unit = first_window.unit;
            let category = first_window.category;
            if ws
                .iter()
                .any(|(w, _)| w.unit != unit || w.category != category)
            {
                not_comparable.push(NotComparableNote {
                    surface: *surface,
                    reason: "different units",
                });
                continue;
            }
            let rem: Vec<u8> = ws.iter().filter_map(|(w, _)| w.remaining_pct()).collect();
            let summed = if unit != WindowUnit::Percent
                && ws
                    .iter()
                    .all(|(w, lg)| !lg && w.used.is_some() && w.limit.is_some())
            {
                Some((
                    ws.iter().map(|(w, _)| w.used.unwrap_or(0)).sum(),
                    ws.iter().map(|(w, _)| w.limit.unwrap_or(0)).sum(),
                ))
            } else {
                None
            };
            comparable.push(ComparableRollup {
                surface: *surface,
                window_id: id,
                label: first_window.label.clone(),
                unit,
                accounts: ws.len(),
                min_remaining_pct: rem.iter().copied().min().unwrap_or(0),
                max_remaining_pct: rem.iter().copied().max().unwrap_or(0),
                summed,
                last_good_count: ws.iter().filter(|(_, lg)| *lg).count(),
                not_visible: not_visible.get(surface).copied().unwrap_or(0),
            });
        }
        for (surface, ids) in seen_single {
            if !comparable.iter().any(|c| c.surface == surface) {
                let _ = ids;
                not_comparable.push(NotComparableNote {
                    surface,
                    reason: "single account or different windows",
                });
            }
        }
        let health = if counts.accounts == 0 {
            HealthWord::Empty
        } else if counts.enabled > 0
            && (blocked == counts.enabled || counts.failed == counts.enabled)
        {
            HealthWord::Blocked
        } else if counts.exhausted > 0
            || counts.failed > 0
            || counts.stale * 2 >= counts.enabled.max(1)
        {
            HealthWord::Degraded
        } else if counts.warnings > 0
            || counts.stale > 0
            || counts.unresolved_identity > 0
            || counts.unsupported > 0
        {
            HealthWord::Attention
        } else {
            HealthWord::Healthy
        };
        Self {
            health,
            counts,
            comparable,
            not_comparable,
            stale_sources,
            failed_sources,
        }
    }

    #[doc = r" `12 accounts · 11 enabled · 1 disabled · 8 providers`."]
    pub fn counts_line(&self) -> String {
        let c = &self.counts;
        let mut parts = vec![format!("{} account{}", c.accounts, plural(c.accounts))];
        parts.push(format!("{} enabled", c.enabled));
        if c.disabled > 0 {
            parts.push(format!("{} disabled", c.disabled));
        }
        parts.push(format!("{} provider{}", c.providers, plural(c.providers)));
        parts.join(" · ")
    }

    #[doc = r" `4 warnings · 1 exhausted · 2 stale · …` (only non-zero parts)."]
    pub fn issues_line(&self) -> String {
        let c = &self.counts;
        let mut parts = vec![];
        if c.warnings > 0 {
            parts.push(format!("{} warning{}", c.warnings, plural(c.warnings)));
        }
        if c.exhausted > 0 {
            parts.push(format!("{} exhausted", c.exhausted));
        }
        if c.stale > 0 {
            parts.push(format!("{} stale", c.stale));
        }
        if c.refreshing > 0 {
            parts.push(format!("{} refreshing", c.refreshing));
        }
        if c.failed > 0 {
            parts.push(format!("{} failed", c.failed));
        }
        if c.unsupported > 0 {
            parts.push(format!("{} quota not visible", c.unsupported));
        }
        if c.unresolved_identity > 0 {
            parts.push(format!(
                "{} identit{} unresolved",
                c.unresolved_identity,
                if c.unresolved_identity == 1 {
                    "y"
                } else {
                    "ies"
                }
            ));
        }
        if parts.is_empty() {
            "no warnings".into()
        } else {
            parts.join(" · ")
        }
    }
}

#[doc = r" Return the plural suffix for a count."]
pub fn plural(n: usize) -> &'static str {
    if n == 1 { "" } else { "s" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quota_status_thresholds() {
        assert_eq!(QuotaStatus::from_pct(10), QuotaStatus::Available);
        assert_eq!(QuotaStatus::from_pct(75), QuotaStatus::Warning);
        assert_eq!(QuotaStatus::from_pct(100), QuotaStatus::Exhausted);
        let w = QuotaWindow::counted(
            "credits",
            "Credits",
            WindowCategory::Other,
            WindowUnit::Credits,
            1240,
            5000,
        );
        assert_eq!(w.value_label(), "1,240 / 5,000 credits");
        assert_eq!(w.used_pct, Some(24));
    }

    #[test]
    fn empty_registry_is_empty_health() {
        let s = OverallSummary::compute(&[]);
        assert_eq!(s.health, HealthWord::Empty);
        assert_eq!(s.counts_line(), "0 accounts · 0 enabled · 0 providers");
    }
}
