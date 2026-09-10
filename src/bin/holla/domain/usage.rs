//! Usage memory (HP02): bounded, decaying frecency per action, learned
//! query→action choices, a versioned serialised form with atomic-style
//! merge, and an opt-out that performs neither reads nor writes. The store
//! is deterministic over the fixture clock; production adapters persist the
//! same text at `$XDG_CACHE_HOME/holla/frecency.json`.

use std::collections::BTreeSet;

pub const MAX_USES: usize = 20;
pub const HALF_LIFE_DAYS: f64 = 10.0;
pub const STALE_DAYS: i64 = 90;
pub const RECENT_THRESHOLD: f64 = 0.05;
pub const RECENT_MAX: usize = 5;
/// Frecency may add at most a quarter of one text bucket to a match.
pub const BOOST_CAP: f64 = 0.25;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionUse {
    pub item: String,
    /// `None` counts as global use on the host.
    pub path: Option<String>,
    pub host: String,
    /// Newest last, at most `MAX_USES`.
    pub stamps: Vec<i64>,
}

impl ActionUse {
    pub fn count(&self) -> u32 {
        self.stamps.len() as u32
    }
    pub fn last_secs(&self) -> i64 {
        self.stamps.last().copied().unwrap_or(0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryChoice {
    /// Normalised: trimmed, lowercased, whitespace collapsed.
    pub query: String,
    pub item: String,
    pub host: String,
    pub last: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageStore {
    pub actions: Vec<ActionUse>,
    pub queries: Vec<QueryChoice>,
    /// `HOLLA_NO_HISTORY=1`: nothing is read or written and nothing learns.
    pub enabled: bool,
    pub load_error: Option<String>,
    pub save_error: Option<String>,
    pub saves: u32,
    pub loaded_from: Option<&'static str>,
}

impl Default for UsageStore {
    fn default() -> Self {
        Self {
            actions: vec![],
            queries: vec![],
            enabled: true,
            load_error: None,
            save_error: None,
            saves: 0,
            loaded_from: None,
        }
    }
}

pub fn normalize_query(q: &str) -> String {
    q.split_whitespace()
        .map(|w| w.to_lowercase())
        .collect::<Vec<_>>()
        .join(" ")
}

impl UsageStore {
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }

    fn entry(&mut self, item: &str, path: Option<&str>, host: &str) -> &mut ActionUse {
        if let Some(i) = self
            .actions
            .iter()
            .position(|u| u.item == item && u.path.as_deref() == path && u.host == host)
        {
            return &mut self.actions[i];
        }
        self.actions.push(ActionUse {
            item: item.into(),
            path: path.map(str::to_owned),
            host: host.into(),
            stamps: vec![],
        });
        self.actions.last_mut().unwrap()
    }

    /// Record a use (path-scoped when `path` is given). Recorded before the
    /// result is known, so failed invocations count too.
    pub fn used(&mut self, item: &str, path: Option<&str>, host: &str, now_secs: i64) {
        if !self.enabled {
            return;
        }
        let e = self.entry(item, path, host);
        e.stamps.push(now_secs);
        e.stamps.sort();
        if e.stamps.len() > MAX_USES {
            let drop = e.stamps.len() - MAX_USES;
            e.stamps.drain(..drop);
        }
    }

    /// Seed a fixture entry with `count` uses ending at `last_secs`.
    pub fn seed(&mut self, item: &str, path: Option<&str>, host: &str, count: u32, last_secs: i64) {
        let e = self.entry(item, path, host);
        e.stamps = (0..count.min(MAX_USES as u32) as i64)
            .map(|i| last_secs - (count as i64 - 1 - i) * 600)
            .collect();
    }

    pub fn learn_query(&mut self, query: &str, item: &str, host: &str, now_secs: i64) {
        if !self.enabled {
            return;
        }
        let q = normalize_query(query);
        if q.is_empty() {
            return;
        }
        self.queries.retain(|c| !(c.query == q && c.host == host));
        self.queries.push(QueryChoice {
            query: q,
            item: item.into(),
            host: host.into(),
            last: now_secs,
        });
    }

    pub fn learned_for(&self, query: &str, host: &str) -> Option<&str> {
        let q = normalize_query(query);
        self.queries
            .iter()
            .find(|c| c.query == q && c.host == host)
            .map(|c| c.item.as_str())
    }

    pub fn find(&self, item: &str, path: Option<&str>, host: &str) -> Option<&ActionUse> {
        self.actions
            .iter()
            .find(|u| u.item == item && u.path.as_deref() == path && u.host == host)
    }

    /// Decayed use weight: each use counts `0.5^(age / 10 days)`; a future
    /// stamp (clock skew) counts as now. The sum is diminished so frequent
    /// use saturates instead of dominating.
    pub fn frecency(&self, item: &str, path: Option<&str>, host: &str, now_secs: i64) -> f64 {
        let Some(u) = self.find(item, path, host) else {
            return 0.0;
        };
        let raw: f64 = u
            .stamps
            .iter()
            .map(|s| {
                let age_days = (now_secs - s).max(0) as f64 / 86_400.0;
                0.5f64.powf(age_days / HALF_LIFE_DAYS)
            })
            .sum();
        raw / (raw + 1.0)
    }

    /// Frecency across the path-scoped and global records of an item.
    pub fn frecency_any(&self, item: &str, path: &str, host: &str, now_secs: i64) -> f64 {
        let here = self.frecency(item, Some(path), host, now_secs);
        let global = self.frecency(item, None, host, now_secs);
        here.max(global)
    }

    /// Items recently used here: above the threshold, at most five,
    /// ordered by score then item id for deterministic ties.
    pub fn recent(&self, path: &str, host: &str, now_secs: i64) -> Vec<(String, f64)> {
        let items: BTreeSet<&str> = self
            .actions
            .iter()
            .filter(|u| u.host == host && (u.path.as_deref() == Some(path) || u.path.is_none()))
            .map(|u| u.item.as_str())
            .collect();
        let mut v: Vec<(String, f64)> = items
            .into_iter()
            .map(|i| (i.to_owned(), self.frecency_any(i, path, host, now_secs)))
            .filter(|(_, s)| *s > RECENT_THRESHOLD)
            .collect();
        v.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });
        v.truncate(RECENT_MAX);
        v
    }

    /// Drop stamps older than 90 days, empty records, and learned queries
    /// whose action has no remaining use.
    pub fn prune(&mut self, now_secs: i64) {
        let cutoff = now_secs - STALE_DAYS * 86_400;
        for u in &mut self.actions {
            u.stamps.retain(|s| *s >= cutoff);
        }
        self.actions.retain(|u| !u.stamps.is_empty());
        let alive: BTreeSet<&str> = self.actions.iter().map(|u| u.item.as_str()).collect();
        self.queries
            .retain(|q| alive.contains(q.item.as_str()) && q.last >= cutoff);
    }

    /// Forget every learned signal for one item.
    pub fn forget(&mut self, item: &str) {
        self.actions.retain(|u| u.item != item);
        self.queries.retain(|q| q.item != item);
    }

    // -------------------------------------------------------- persistence

    /// Versioned JSON; deterministic ordering.
    pub fn serialize(&self) -> String {
        let mut actions: Vec<&ActionUse> = self.actions.iter().collect();
        actions.sort_by(|a, b| {
            a.item
                .cmp(&b.item)
                .then_with(|| a.host.cmp(&b.host))
                .then_with(|| a.path.cmp(&b.path))
        });
        let a: Vec<String> = actions
            .iter()
            .map(|u| {
                format!(
                    "{{\"item\":{},\"path\":{},\"host\":{},\"stamps\":[{}]}}",
                    js(&u.item),
                    u.path.as_deref().map(js).unwrap_or("null".into()),
                    js(&u.host),
                    u.stamps
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                )
            })
            .collect();
        let mut queries: Vec<&QueryChoice> = self.queries.iter().collect();
        queries.sort_by(|a, b| a.query.cmp(&b.query).then_with(|| a.host.cmp(&b.host)));
        let q: Vec<String> = queries
            .iter()
            .map(|c| {
                format!(
                    "{{\"query\":{},\"item\":{},\"host\":{},\"last\":{}}}",
                    js(&c.query),
                    js(&c.item),
                    js(&c.host),
                    c.last
                )
            })
            .collect();
        format!(
            "{{\"v\":1,\"actions\":[{}],\"queries\":[{}]}}",
            a.join(","),
            q.join(",")
        )
    }

    /// Load persisted text. Missing, corrupt or unknown-schema input is an
    /// empty store with the reason kept for the status line; `now` clamps
    /// stamps from the future.
    pub fn load(text: Option<&str>, now_secs: i64) -> Self {
        let mut store = Self::default();
        let Some(text) = text else {
            return store;
        };
        use crate::domain::manifest::{Json, parse_json};
        let json = match parse_json(text) {
            Ok(j) => j,
            Err(e) => {
                store.load_error = Some(format!("frecency.json unreadable: {e} · starting empty"));
                return store;
            }
        };
        match json.get("v") {
            Some(Json::Num(n)) if *n as i64 == 1 => {}
            Some(Json::Num(n)) => {
                store.load_error = Some(format!(
                    "frecency.json schema {} unknown · starting empty",
                    *n as i64
                ));
                return store;
            }
            _ => {
                store.load_error =
                    Some("frecency.json has no schema version · starting empty".into());
                return store;
            }
        }
        if let Some(Json::Arr(items)) = json.get("actions") {
            for it in items {
                let (Some(item), Some(host)) = (
                    it.get("item").and_then(Json::as_str),
                    it.get("host").and_then(Json::as_str),
                ) else {
                    continue;
                };
                let path = it.get("path").and_then(Json::as_str).map(str::to_owned);
                let mut stamps: Vec<i64> = match it.get("stamps") {
                    Some(Json::Arr(s)) => s
                        .iter()
                        .filter_map(|v| match v {
                            Json::Num(n) => Some((*n as i64).min(now_secs)),
                            _ => None,
                        })
                        .collect(),
                    _ => vec![],
                };
                stamps.sort();
                if stamps.len() > MAX_USES {
                    let drop = stamps.len() - MAX_USES;
                    stamps.drain(..drop);
                }
                store.actions.push(ActionUse {
                    item: item.into(),
                    path,
                    host: host.into(),
                    stamps,
                });
            }
        }
        if let Some(Json::Arr(items)) = json.get("queries") {
            for it in items {
                let (Some(query), Some(item), Some(host)) = (
                    it.get("query").and_then(Json::as_str),
                    it.get("item").and_then(Json::as_str),
                    it.get("host").and_then(Json::as_str),
                ) else {
                    continue;
                };
                let last = match it.get("last") {
                    Some(Json::Num(n)) => (*n as i64).min(now_secs),
                    _ => 0,
                };
                store.queries.push(QueryChoice {
                    query: query.into(),
                    item: item.into(),
                    host: host.into(),
                    last,
                });
            }
        }
        store.loaded_from = Some("frecency.json");
        store
    }

    /// Merge another writer's store into this one: stamps union (capped,
    /// newest kept), newest query choice wins. Used before an atomic save
    /// so a concurrent writer's updates are never lost.
    pub fn merge(&mut self, other: &UsageStore) {
        for o in &other.actions {
            let e = self.entry(&o.item, o.path.as_deref(), &o.host);
            let mut all: Vec<i64> = e.stamps.iter().chain(o.stamps.iter()).copied().collect();
            all.sort();
            all.dedup();
            if all.len() > MAX_USES {
                let drop = all.len() - MAX_USES;
                all.drain(..drop);
            }
            e.stamps = all;
        }
        for q in &other.queries {
            match self
                .queries
                .iter_mut()
                .find(|c| c.query == q.query && c.host == q.host)
            {
                Some(c) if c.last < q.last => *c = q.clone(),
                Some(_) => {}
                None => self.queries.push(q.clone()),
            }
        }
    }

    /// Save: merge the on-disk text first, then produce the new text. A
    /// save failure is reported but never changes an action's outcome.
    pub fn save(&mut self, on_disk: Option<&str>, now_secs: i64) -> Result<String, String> {
        if !self.enabled {
            return Err("history disabled (HOLLA_NO_HISTORY=1)".into());
        }
        if let Some(e) = &self.save_error {
            return Err(e.clone());
        }
        if let Some(text) = on_disk {
            let disk = UsageStore::load(Some(text), now_secs);
            if disk.load_error.is_none() {
                self.merge(&disk);
            }
        }
        self.prune(now_secs);
        self.saves += 1;
        Ok(self.serialize())
    }
}

fn js(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

#[cfg(test)]
mod tests {
    use super::*;

    const DAY: i64 = 86_400;

    #[test]
    fn frecency_decays_saturates_and_bounds_uses() {
        let mut s = UsageStore::default();
        let now = 1_000_000 * DAY;
        s.used("a", Some("/p"), "h", now);
        let one = s.frecency("a", Some("/p"), "h", now);
        assert!((one - 0.5).abs() < 1e-9, "one fresh use: 1/(1+1)");
        s.used("a", Some("/p"), "h", now - 10 * DAY);
        let two = s.frecency("a", Some("/p"), "h", now);
        assert!(
            (two - (1.5 / 2.5)).abs() < 1e-9,
            "a ten-day-old use counts half"
        );
        for i in 0..40 {
            s.used("a", Some("/p"), "h", now - i);
        }
        assert_eq!(
            s.find("a", Some("/p"), "h").unwrap().count(),
            MAX_USES as u32
        );
        assert!(s.frecency("a", Some("/p"), "h", now) < 1.0);
        // clock skew: a future stamp never inflates the score
        s.seed("f", None, "h", 1, now + 400 * DAY);
        let loaded = UsageStore::load(Some(&s.serialize()), now);
        assert_eq!(loaded.find("f", None, "h").unwrap().last_secs(), now);
        // a stale use falls below the recent threshold and is pruned
        s.seed("old", Some("/p"), "h", 1, now - 60 * DAY);
        assert!(s.frecency("old", Some("/p"), "h", now) < RECENT_THRESHOLD);
        s.seed("gone", Some("/p"), "h", 3, now - 95 * DAY);
        s.learn_query("stale q", "gone", "h", now - 95 * DAY);
        s.prune(now);
        assert!(s.find("gone", Some("/p"), "h").is_none());
        assert!(s.learned_for("stale q", "h").is_none());
    }

    #[test]
    fn recent_is_bounded_positive_and_deterministic() {
        let mut s = UsageStore::default();
        let now = 1_000_000 * DAY;
        for (i, item) in ["b", "a", "c", "d", "e", "f", "g"].iter().enumerate() {
            s.seed(item, Some("/p"), "h", 3, now - i as i64 * 60);
        }
        s.seed("tie1", Some("/p"), "h", 1, now);
        s.seed("tie0", Some("/p"), "h", 1, now);
        s.seed("weak", Some("/p"), "h", 1, now - 80 * DAY);
        s.seed("elsewhere", Some("/q"), "h", 9, now);
        let r = s.recent("/p", "h", now);
        assert_eq!(r.len(), RECENT_MAX);
        assert!(r.iter().all(|(_, sc)| *sc > RECENT_THRESHOLD));
        assert!(!r.iter().any(|(i, _)| i == "weak" || i == "elsewhere"));
        let ids: Vec<&str> = r.iter().map(|(i, _)| i.as_str()).collect();
        assert_eq!(ids, vec!["b", "a", "c", "d", "e"], "score then id: {ids:?}");
        // ties resolve by id
        let mut t = UsageStore::default();
        t.seed("tie1", Some("/p"), "h", 1, now);
        t.seed("tie0", Some("/p"), "h", 1, now);
        let ids: Vec<String> = t
            .recent("/p", "h", now)
            .into_iter()
            .map(|(i, _)| i)
            .collect();
        assert_eq!(ids, vec!["tie0", "tie1"]);
    }

    #[test]
    fn learned_queries_normalise_and_survive_restart() {
        let mut s = UsageStore::default();
        let now = 1_000_000 * DAY;
        s.used("docker.cleanup", None, "h", now);
        s.learn_query("  Docker   CLEAN ", "docker.cleanup", "h", now);
        assert_eq!(s.learned_for("docker clean", "h"), Some("docker.cleanup"));
        assert_eq!(s.learned_for("docker clean", "other"), None);
        s.learn_query("docker clean", "docker.stop_all", "h", now + 1);
        assert_eq!(s.learned_for("docker clean", "h"), Some("docker.stop_all"));
        assert_eq!(normalize_query("Ünï  code"), "ünï code");
        let text = s.serialize();
        let again = UsageStore::load(Some(&text), now + 2);
        assert_eq!(
            again.learned_for("docker clean", "h"),
            Some("docker.stop_all")
        );
        assert_eq!(again.serialize(), text, "round trip is exact");
        assert_eq!(again.loaded_from, Some("frecency.json"));
    }

    #[test]
    fn corruption_versions_merge_and_opt_out() {
        let now = 1_000_000 * DAY;
        assert!(
            UsageStore::load(Some("{"), now)
                .load_error
                .unwrap()
                .contains("unreadable")
        );
        assert!(
            UsageStore::load(Some(r#"{"v":7}"#), now)
                .load_error
                .unwrap()
                .contains("schema 7")
        );
        assert!(
            UsageStore::load(Some(r#"{"actions":[]}"#), now)
                .load_error
                .is_some()
        );
        assert!(UsageStore::load(None, now).load_error.is_none());
        // two writers: the save merges what is on disk first
        let mut a = UsageStore::default();
        a.used("x", Some("/p"), "h", now - 5);
        let mut b = UsageStore::default();
        b.used("x", Some("/p"), "h", now - 3);
        b.used("y", None, "h", now - 2);
        b.learn_query("q", "y", "h", now - 2);
        let disk = b.serialize();
        let saved = a.save(Some(&disk), now).unwrap();
        let merged = UsageStore::load(Some(&saved), now);
        assert_eq!(
            merged.find("x", Some("/p"), "h").unwrap().stamps,
            vec![now - 5, now - 3]
        );
        assert!(merged.find("y", None, "h").is_some());
        assert_eq!(merged.learned_for("q", "h"), Some("y"));
        // a save error is reported, not fatal, and changes nothing else
        let mut c = UsageStore {
            save_error: Some("EACCES".into()),
            ..Default::default()
        };
        c.used("z", None, "h", now);
        assert!(c.save(None, now).is_err());
        assert_eq!(c.find("z", None, "h").unwrap().count(), 1);
        // opt-out performs neither reads nor writes nor learning
        let mut off = UsageStore::disabled();
        off.used("z", None, "h", now);
        off.learn_query("q", "z", "h", now);
        assert!(off.actions.is_empty() && off.queries.is_empty());
        assert!(
            off.save(None, now)
                .unwrap_err()
                .contains("HOLLA_NO_HISTORY")
        );
    }
}
