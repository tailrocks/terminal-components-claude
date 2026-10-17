//! Search and ranking over the catalogue (CONCEPT §6.3, §6.6, §9): one query
//! across actions and resources, fuzzy matching over labels, keywords and
//! resource names, exact aliases that beat every learned signal, scope
//! tokens, and a ranking order that explains itself.

use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

struct SearchText {
    text: String,
    origins: Vec<(Range<usize>, Range<usize>)>,
}

impl SearchText {
    fn new(source: &str, lowercase: bool) -> Self {
        let text = if lowercase {
            source.to_lowercase()
        } else {
            source.to_owned()
        };
        let mut chars = text.char_indices().peekable();
        let mut origins = Vec::new();
        for (start, grapheme) in source.grapheme_indices(true) {
            let mapped_start = chars.peek().map_or(text.len(), |(offset, _)| *offset);
            let count = if lowercase {
                grapheme.chars().map(|c| c.to_lowercase().count()).sum()
            } else {
                grapheme.chars().count()
            };
            for _ in 0..count {
                chars.next();
            }
            let mapped_end = chars.peek().map_or(text.len(), |(offset, _)| *offset);
            origins.push((mapped_start..mapped_end, start..start + grapheme.len()));
        }
        Self { text, origins }
    }

    fn original_range(&self, range: Range<usize>) -> Range<usize> {
        let first = self
            .origins
            .partition_point(|(mapped, _)| mapped.end <= range.start);
        let end = self
            .origins
            .partition_point(|(mapped, _)| mapped.start < range.end);
        self.origins[first].1.start..self.origins[end - 1].1.end
    }

    fn matched_offsets(&self, range: Range<usize>) -> Vec<usize> {
        self.origins
            .iter()
            .filter(|(mapped, _)| mapped.start < range.end && range.start < mapped.end)
            .map(|(_, original)| original.start)
            .collect()
    }
}

pub fn fuzzy(label: &str, word: &str) -> Option<(u32, Vec<usize>)> {
    if word.is_empty() {
        return Some((0, vec![]));
    }
    let l = SearchText::new(label, true);
    let w = word.to_lowercase();
    if l.text.starts_with(&w) {
        return Some((0, l.matched_offsets(0..w.len())));
    }
    if let Some(p) = l.text.find(&w) {
        let boundary = p == 0 || matches!(l.text.as_bytes()[p - 1], b'_' | b'.');
        return Some((
            if boundary { 10 } else { 30 },
            l.matched_offsets(p..p + w.len()),
        ));
    }
    let mut matched = Vec::new();
    let mut chars = l.text.char_indices();
    for wc in w.chars() {
        let (offset, ch) = chars.find(|(_, ch)| *ch == wc)?;
        let original = l.original_range(offset..offset + ch.len_utf8()).start;
        if matched.last() != Some(&original) {
            matched.push(original);
        }
    }
    Some((60 + (matched.last().copied().unwrap_or(0) as u32), matched))
}

use crate::action::{Item, Signal};
use crate::context::Scope;

/// One ranked row: the item index, the score, the matched byte offsets in
/// the label, and the reason to show.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ranked {
    pub index: usize,
    pub score: i64,
    pub matched: Vec<usize>,
    pub reason: String,
    /// `alias gp` when an alias matched exactly.
    pub tag: Option<String>,
}

/// The parsed query: free text plus an optional scope token.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Query {
    pub text: String,
    pub scope: Option<ScopeToken>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeToken {
    Here,
    Parent,
    Children,
    System,
    All,
}

impl Query {
    pub fn parse(raw: &str) -> Self {
        let mut scope = None;
        let mut words = vec![];
        for w in raw.split_whitespace() {
            match w.to_lowercase().as_str() {
                "@here" | "@current" => scope = Some(ScopeToken::Here),
                "@parent" | "@ancestors" => scope = Some(ScopeToken::Parent),
                "@children" | "@child" => scope = Some(ScopeToken::Children),
                "@system" | "@host" => scope = Some(ScopeToken::System),
                "@all" => scope = Some(ScopeToken::All),
                _ => words.push(w),
            }
        }
        Self {
            text: words.join(" "),
            scope,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.text.trim().is_empty()
    }
}

/// Which scope filter applies after the query token and the readout:
/// the scope, and whether it is strict (a typed token) or the readout's
/// bias.
pub fn effective_scope(readout: Scope, token: Option<ScopeToken>) -> (Option<Scope>, bool) {
    match token {
        Some(ScopeToken::All) => (None, true),
        Some(ScopeToken::Here) => (Some(Scope::Here), true),
        Some(ScopeToken::Parent) => (Some(Scope::Parent), true),
        Some(ScopeToken::Children) => (Some(Scope::Children), true),
        Some(ScopeToken::System) => (Some(Scope::System), true),
        None => (Some(readout), false),
    }
}

/// A row belongs to the scope. `Here` without a token is a bias: urgent
/// rows, query matches, Explore and activity rows cross it. A typed token
/// or a narrowed readout filters exactly.
fn in_scope(item: &Item, scope: Option<Scope>, strict: bool, has_query: bool) -> bool {
    match scope {
        None => true,
        Some(Scope::Here) if !strict => {
            item.scope.direction == Scope::Here
                || item.signal() >= Signal::Urgency
                || has_query
                || item.kind == crate::action::Kind::Explore
                || item.kind == crate::action::Kind::Activity
        }
        Some(s) => item.scope.direction == s,
    }
}

/// Matching score for a query word against an item: lower is better;
/// `None` when the word does not match at all.
fn word_score(item: &Item, word: &str) -> Option<(u32, Vec<usize>)> {
    let mut best: Option<(u32, Vec<usize>)> = None;
    let mut consider = |score: u32, matched: Vec<usize>| {
        if best.as_ref().is_none_or(|b| score < b.0) {
            best = Some((score, matched));
        }
    };
    if let Some((s, m)) = fuzzy(&item.label, word) {
        consider(s, m);
    }
    let lw = word.to_lowercase();
    for k in &item.keywords {
        let k = k.to_lowercase();
        if k == lw {
            consider(5, vec![]);
        } else if k.starts_with(&lw) {
            consider(12, vec![]);
        } else if k.contains(&lw) {
            consider(35, vec![]);
        }
    }
    if item.kind.label().starts_with(&lw) || item.group.to_lowercase().starts_with(&lw) {
        consider(40, vec![]);
    }
    if item.scope.word.starts_with(&lw) {
        consider(45, vec![]);
    }
    best
}

/// Rank the catalogue for a query in a scope. Empty queries rank by the
/// recommendation order; non-empty queries require every word to match.
pub fn search(items: &[Item], raw: &str, readout: Scope, group: Option<&str>) -> Vec<Ranked> {
    search_with(items, raw, readout, group, None)
}

/// `search` with the item remembered for this exact query (HP02): it ranks
/// first among the rows that still match the text, below an alias, and
/// never authorizes anything.
pub fn search_with(
    items: &[Item],
    raw: &str,
    readout: Scope,
    group: Option<&str>,
    learned: Option<&str>,
) -> Vec<Ranked> {
    let q = Query::parse(raw);
    let (scope, strict) = effective_scope(readout, q.scope);
    let has_query = !q.is_empty();
    let text = q.text.trim().to_lowercase();
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut out = vec![];
    for (i, item) in items.iter().enumerate() {
        if item.hidden && !has_query {
            continue;
        }
        // an exact alias is deterministic user intent: it crosses scope
        let alias_hit = has_query
            && (item.aliases.iter().any(|a| a.eq_ignore_ascii_case(&text))
                || (group.is_some()
                    && item
                        .page_aliases
                        .iter()
                        .any(|a| a.eq_ignore_ascii_case(&text))));
        if let Some(g) = group {
            if item.group != g || item.kind == crate::action::Kind::Explore {
                continue;
            }
        } else if !alias_hit && !in_scope(item, scope, strict, has_query) {
            continue;
        }
        let mut score: i64 = 0;
        let mut matched = vec![];
        let mut tag = None;
        let mut reason = item
            .top_reason()
            .map(|r| r.text.clone())
            .unwrap_or_default();
        if item.used_here > 0 && item.signal() > Signal::LocalUse {
            reason = format!(
                "{reason} · used {} here",
                if item.used_here == 1 {
                    "once".to_owned()
                } else {
                    format!("{} times", item.used_here)
                }
            );
        }
        if has_query {
            if alias_hit {
                tag = Some(format!("alias {text}"));
                reason = format!("alias {text}");
                score += 5_000_000;
            } else {
                let mut total = 0u32;
                for w in &words {
                    match word_score(item, w) {
                        Some((s, m)) => {
                            total += s;
                            if matched.is_empty() {
                                matched = m;
                            }
                        }
                        None => {
                            total = u32::MAX;
                            break;
                        }
                    }
                }
                if total == u32::MAX {
                    continue;
                }
                // match quality comes first: a prefix or substring always
                // beats a scattered subsequence, whatever the live signal;
                // the whole phrase inside the label is its own top bucket,
                // above any set of exact keyword hits
                let phrase = item.label.to_lowercase().contains(&text);
                // a measured suffix (" · 900 MiB") is not part of the name
                let core = item
                    .label
                    .split(" · ")
                    .next()
                    .unwrap_or(&item.label)
                    .to_lowercase();
                let exact = core == text;
                if phrase {
                    // among phrase matches the label closest to the query is
                    // the intended one
                    total =
                        (core.chars().count().saturating_sub(text.chars().count())).min(199) as u32;
                }
                // the typed name itself is decisive: no learned signal moves
                // a longer label above it
                let bucket: i64 = if exact {
                    10
                } else if phrase {
                    9
                } else {
                    match total / words.len().max(1) as u32 {
                        0..=9 => 8,
                        10..=19 => 7,
                        20..=39 => 6,
                        40..=49 => 4,
                        _ => 2,
                    }
                };
                score += bucket * 100_000 + (200 - (total as i64).min(199));
            }
        }
        // the ranking order of CONCEPT §9 as tiers above the text score
        let tier = match item.signal() {
            Signal::Pin => 7,
            Signal::Urgency => 6,
            Signal::Context => 5,
            Signal::LocalUse => 3,
            Signal::GlobalUse => 2,
            Signal::Default => 1,
        };
        // a pin, a remembered query and frecency all stay inside the match
        // bucket: a better text match is never displaced by a learned signal
        if item.pinned {
            score += 90_000;
            reason = format!("pinned here · {reason}");
        }
        if has_query && learned == Some(item.id.as_str()) {
            // the remembered choice for this exact query leads every text
            // match (it still had to match) and yields only to an alias
            score += 950_000;
            reason = format!("remembered for “{}” · {reason}", text);
        }
        if item.frecency > 0.0 {
            // bounded: at most a quarter of one match-quality step
            score += (item.frecency.clamp(0.0, 1.0) * crate::usage::BOOST_CAP * 100_000.0) as i64;
        }
        score += tier * 5_000;
        score += (item.used_here.min(50) as i64) * 100;
        score += (item.used_anywhere.min(50) as i64) * 10;
        if item.scope.direction == Scope::Here {
            score += 500;
        }
        if item.hidden {
            score -= 2_000_000;
            reason = format!("hidden here · {reason}");
        }
        out.push(Ranked {
            index: i,
            score,
            matched,
            reason,
            tag,
        });
    }
    out.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| items[a.index].label.cmp(&items[b.index].label))
    });
    out
}

/// The full explanation of a row's rank, for "Why is this here?".
pub fn explain(item: &Item, cwd_short: &str) -> Vec<(String, String)> {
    let mut rows = vec![];
    for r in &item.reasons {
        let label = match r.signal {
            Signal::Pin => "pinned",
            Signal::Urgency => "live state",
            Signal::Context => "context",
            Signal::LocalUse => "used here",
            Signal::GlobalUse => "used elsewhere",
            Signal::Default => "default",
        };
        rows.push((label.to_owned(), r.text.clone()));
    }
    if item.used_here > 0 {
        rows.push((
            "used here".into(),
            format!("{} times in {cwd_short}", item.used_here),
        ));
    }
    if item.used_anywhere > 0 {
        rows.push((
            "used anywhere".into(),
            format!("{} times on this host", item.used_anywhere),
        ));
    }
    if !item.aliases.is_empty() {
        rows.push(("aliases".into(), item.aliases.join(", ")));
    }
    rows.push((
        "risk".into(),
        format!(
            "{} · {} · never changes the rank",
            item.risk.label(),
            item.confirmation.label()
        ),
    ));
    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::{Kind, ResultType, Risk};
    use crate::context::ScopeTag;

    fn items() -> Vec<Item> {
        vec![
            Item::new(
                "git.pull",
                "Pull",
                Kind::Git,
                ResultType::Action,
                ScopeTag::here("/w"),
            )
            .aliases(&["gp"])
            .keywords(&["sync", "fetch"])
            .reason(Signal::Urgency, "branch is 3 commits behind"),
            Item::new(
                "git.push",
                "Push",
                Kind::Git,
                ResultType::Action,
                ScopeTag::here("/w"),
            )
            .reason(Signal::Context, "1 commit ahead"),
            Item::new(
                "docker.clean",
                "Clean Docker completely",
                Kind::Docker,
                ResultType::Action,
                ScopeTag::host("devbox"),
            )
            .risk(Risk::Destructive)
            .aliases(&["dc"])
            .keywords(&["docker cleanup", "prune", "remove everything"])
            .reason(Signal::LocalUse, "used 9 times on devbox"),
            Item::new(
                "docker.stop",
                "Stop all containers",
                Kind::Docker,
                ResultType::Action,
                ScopeTag::host("devbox"),
            )
            .keywords(&["docker"])
            .reason(Signal::Default, "available on this host"),
            Item::new(
                "disk.usage",
                "Analyze disk usage",
                Kind::Disk,
                ResultType::Flow,
                ScopeTag::here("/w"),
            )
            .aliases(&["du"])
            .keywords(&["why disk full", "space", "largest"])
            .reason(Signal::Default, "available"),
        ]
    }

    #[test]
    fn empty_query_keeps_here_and_urgent_rows_and_ranks_by_signal() {
        let it = items();
        let r = search(&it, "", Scope::Here, None);
        let labels: Vec<&str> = r.iter().map(|x| it[x.index].label.as_str()).collect();
        assert_eq!(labels[0], "Pull", "{labels:?}");
        assert!(
            !labels.contains(&"Stop all containers"),
            "host rows stay out of Here without a query"
        );
        assert!(!labels.contains(&"Clean Docker completely"));
    }

    #[test]
    fn aliases_beat_everything_and_carry_a_tag() {
        let it = items();
        let r = search(&it, "dc", Scope::Here, None);
        assert_eq!(it[r[0].index].id, "docker.clean");
        assert_eq!(r[0].tag.as_deref(), Some("alias dc"));
        let r = search(&it, "gp", Scope::System, None);
        assert_eq!(
            it[r[0].index].id, "git.pull",
            "an alias crosses the scope filter"
        );
    }

    #[test]
    fn intent_phrases_match_keywords_and_scope_tokens_filter() {
        let it = items();
        let r = search(&it, "why disk full", Scope::Here, None);
        assert_eq!(it[r[0].index].id, "disk.usage");
        let r = search(&it, "docker", Scope::Here, None);
        let ids: Vec<&str> = r.iter().map(|x| it[x.index].id.as_str()).collect();
        assert!(
            ids.contains(&"docker.clean") && ids.contains(&"docker.stop"),
            "{ids:?}"
        );
        assert_eq!(ids[0], "docker.clean", "frequent use ranks above default");
        let r = search(&it, "@system docker", Scope::Here, None);
        assert!(
            r.iter()
                .all(|x| it[x.index].scope.direction == Scope::System)
        );
        let r = search(&it, "@here docker", Scope::Here, None);
        assert!(r.is_empty());
    }

    #[test]
    fn risk_never_demotes_a_strong_match() {
        let it = items();
        let r = search(&it, "docker clean", Scope::System, None);
        assert_eq!(it[r[0].index].id, "docker.clean");
        assert_eq!(it[r[0].index].risk, Risk::Destructive);
    }
}
