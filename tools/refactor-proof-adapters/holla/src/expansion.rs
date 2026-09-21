//! Finite HO scenario expansion: every register row becomes a program.
//!
//! Source: `data/holla-scenarios.tsv`, a byte copy of
//! `docs/refactoring-plan/holla-scenarios.tsv` (135 rows). Actions split on
//! top-level `;`. Structured verbs (`NEW/DRAW/TICKS/TYPE/PASTE/OPEN/RUN/
//! SETTLE/SCAN/SWEEP/ROUTE/KEY`), bare key tokens, and explicit `resize`
//! size lists become typed actions; every other clause is retained verbatim
//! as [`Action::Raw`] so no register text is ever dropped.

use holla_app::Motion;

use crate::branches::{NAMED_BRANCHES, NamedBranch};
use crate::observe::ORACLE_SIZES;
use crate::tsv::Table;
use crate::worlds::intern_world;

/// Frozen scenario register.
const SCENARIOS_TSV: &str = include_str!("../data/holla-scenarios.tsv");

/// One immutable action in an expanded program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// `NEW(world, motion, tick)`.
    New {
        /// Oracle world name.
        world: String,
        /// Motion policy.
        motion: Motion,
        /// Oracle tick ordinal.
        tick: u64,
    },
    /// Draw the current frame.
    Draw,
    /// Deliver `n` virtual ticks.
    Ticks(u32),
    /// Type Unicode scalars.
    Type(String),
    /// Bracketed paste.
    Paste(String),
    /// OPEN(label) helper.
    Open(String),
    /// RUN(label) helper.
    Run(String),
    /// SETTLE(id) helper.
    Settle(String),
    /// SCAN(kind) helper.
    Scan(String),
    /// SWEEP expansion marker.
    Sweep,
    /// ROUTE(file::test) native seed.
    Route {
        /// Source path.
        file: String,
        /// Test symbol.
        test: String,
    },
    /// Key with optional modifiers, stored as source text.
    Key(String),
    /// Explicit bare `resize WxH,...` size list.
    Resize(Vec<(u16, u16)>),
    /// Repeat the constructor in a fresh world.
    RepeatFresh,
    /// Unparsed remainder retained so no clause is dropped.
    Raw(String),
}

/// Expanded scenario program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioProgram {
    /// HO-* identity.
    pub id: String,
    /// Fixture world or native-test marker.
    pub fixture: String,
    /// Declared sizes.
    pub sizes: Vec<(u16, u16)>,
    /// Structured actions.
    pub actions: Vec<Action>,
    /// Named branches attached to this parent.
    pub branches: Vec<String>,
    /// Raw actions column.
    pub raw_actions: String,
}

/// Parse and expand the frozen scenario register.
#[must_use]
pub fn expand_scenarios() -> Vec<ScenarioProgram> {
    let Some(table) = Table::parse(SCENARIOS_TSV) else {
        return Vec::new();
    };
    let mut programs = Vec::with_capacity(table.len());
    let mut index = 0;
    while index < table.len() {
        let Some(id) = table.get(index, "id") else {
            break;
        };
        let fixture = table.get(index, "fixture").unwrap_or("").to_owned();
        let raw_actions = table.get(index, "actions").unwrap_or("").to_owned();
        let sizes = parse_sizes(table.get(index, "sizes").unwrap_or(""));
        let actions = parse_actions(&raw_actions);
        let branches = NAMED_BRANCHES
            .iter()
            .filter(|branch| branch.parent == id)
            .map(|branch| branch.id.to_owned())
            .collect();
        programs.push(ScenarioProgram {
            id: id.to_owned(),
            fixture,
            sizes,
            actions,
            branches,
            raw_actions,
        });
        index = index.saturating_add(1);
    }
    programs
}

/// Scenario IDs in register order.
#[must_use]
pub fn scenario_ids() -> Vec<String> {
    expand_scenarios()
        .into_iter()
        .map(|program| program.id)
        .collect()
}

fn parse_sizes(spec: &str) -> Vec<(u16, u16)> {
    let mut sizes = Vec::new();
    for token in spec.split(',') {
        let token = token.trim();
        if let Some((w, h)) = token.split_once('x')
            && let (Ok(width), Ok(height)) = (w.parse::<u16>(), h.parse::<u16>())
        {
            sizes.push((width, height));
        }
    }
    if sizes.is_empty() {
        ORACLE_SIZES.into_iter().collect()
    } else {
        sizes
    }
}

fn parse_actions(raw: &str) -> Vec<Action> {
    let mut out = Vec::new();
    for part in split_top_level(raw) {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some(inner) = part.strip_prefix("NEW(").and_then(|s| s.strip_suffix(')')) {
            out.push(parse_new(inner));
        } else if part == "DRAW" {
            out.push(Action::Draw);
        } else if let Some(rest) = part
            .strip_prefix("TICKS(")
            .and_then(|s| s.strip_suffix(')'))
        {
            out.push(Action::Ticks(rest.parse().unwrap_or(0)));
        } else if let Some(rest) = part.strip_prefix("TYPE(").and_then(|s| s.strip_suffix(')')) {
            out.push(Action::Type(rest.to_owned()));
        } else if let Some(rest) = part
            .strip_prefix("PASTE(")
            .and_then(|s| s.strip_suffix(')'))
        {
            out.push(Action::Paste(rest.to_owned()));
        } else if let Some(rest) = part.strip_prefix("OPEN(").and_then(|s| s.strip_suffix(')')) {
            out.push(Action::Open(rest.to_owned()));
        } else if let Some(rest) = part.strip_prefix("RUN(").and_then(|s| s.strip_suffix(')')) {
            out.push(Action::Run(rest.to_owned()));
        } else if let Some(rest) = part
            .strip_prefix("SETTLE(")
            .and_then(|s| s.strip_suffix(')'))
        {
            out.push(Action::Settle(rest.to_owned()));
        } else if let Some(rest) = part.strip_prefix("SCAN(").and_then(|s| s.strip_suffix(')')) {
            out.push(Action::Scan(rest.to_owned()));
        } else if part == "SWEEP" || part.starts_with("SWEEP ") || part.starts_with("SWEEP(") {
            out.push(Action::Sweep);
            if part != "SWEEP" {
                out.push(Action::Raw(part.to_owned()));
            }
        } else if let Some(rest) = part
            .strip_prefix("ROUTE(")
            .and_then(|s| s.strip_suffix(')'))
        {
            out.push(parse_route(rest));
        } else if let Some(rest) = part.strip_prefix("KEY(").and_then(|s| s.strip_suffix(')')) {
            out.push(Action::Key(rest.to_owned()));
        } else if let Some(rest) = part.strip_prefix("resize ") {
            out.push(Action::Resize(parse_sizes(rest)));
            out.push(Action::Raw(part.to_owned()));
        } else if part.contains("repeat fresh") {
            out.push(Action::RepeatFresh);
            out.push(Action::Raw(part.to_owned()));
        } else if is_bare_key(part) {
            out.push(Action::Key(part.to_owned()));
        } else {
            out.push(Action::Raw(part.to_owned()));
        }
    }
    out
}

/// Whether a bare clause is a key token: one scalar, a named key, an `F<n>`
/// key, or a `Ctrl/Alt/Shift` chord over those.
fn is_bare_key(part: &str) -> bool {
    if part.contains(' ') || part.contains('\t') {
        return false;
    }
    let mut chars = part.chars();
    if chars.next().is_some() && chars.next().is_none() {
        return true;
    }
    is_key_name(part) || is_chord(part)
}

fn is_key_name(name: &str) -> bool {
    matches!(
        name,
        "Esc"
            | "Escape"
            | "Enter"
            | "Tab"
            | "BackTab"
            | "Space"
            | "Backspace"
            | "Delete"
            | "Up"
            | "Down"
            | "Left"
            | "Right"
            | "Home"
            | "End"
            | "PageUp"
            | "PageDown"
            | "Insert"
    ) || name
        .strip_prefix('F')
        .is_some_and(|digits| !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()))
}

fn is_chord(part: &str) -> bool {
    let Some((mods, key)) = part.rsplit_once('+') else {
        return false;
    };
    if !is_key_name(key) && key.chars().count() != 1 {
        return false;
    }
    !mods.is_empty()
        && mods
            .split('+')
            .all(|m| matches!(m, "Ctrl" | "Alt" | "Shift"))
}

fn parse_new(inner: &str) -> Action {
    let mut bits = inner.split(',');
    let world = bits.next().unwrap_or("").trim();
    let motion_name = bits.next().unwrap_or("").trim();
    let tick = bits
        .next()
        .unwrap_or("0")
        .trim()
        .parse::<u64>()
        .unwrap_or(0);
    Action::New {
        world: intern_world(world).unwrap_or(world).to_owned(),
        motion: motion_from_name(motion_name),
        tick,
    }
}

fn parse_route(inner: &str) -> Action {
    match inner.rsplit_once("::") {
        Some((file, test)) => Action::Route {
            file: file.trim().to_owned(),
            test: test.trim().to_owned(),
        },
        None => Action::Route {
            file: inner.to_owned(),
            test: String::new(),
        },
    }
}

fn motion_from_name(name: &str) -> Motion {
    match name {
        "Paused" | "paused" => Motion::Paused,
        "Reduced" | "reduced" => Motion::Reduced,
        _ => Motion::Full,
    }
}

fn split_top_level(raw: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut depth = 0i32;
    for (index, ch) in raw.char_indices() {
        match ch {
            '(' => depth = depth.saturating_add(1),
            ')' => depth = depth.saturating_sub(1),
            ';' if depth == 0 => {
                out.push(raw.get(start..index).unwrap_or(""));
                start = index.saturating_add(1);
            }
            _ => {}
        }
    }
    if let Some(tail) = raw.get(start..) {
        out.push(tail);
    }
    out
}

/// Branches belonging to `parent`.
#[must_use]
pub fn branches_for(parent: &str) -> Vec<&'static NamedBranch> {
    NAMED_BRANCHES
        .iter()
        .filter(|branch| branch.parent == parent)
        .collect()
}

#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn verbs_keys_resize_and_raw_survive() {
        let actions = parse_actions(
            "NEW(first-use,Paused,40); DRAW; TICKS(10); Esc; Ctrl+G; g; \
             resize 60x15,72x20; SWEEP preview; ROUTE(app_tests_flows.rs::x); \
             run the unchanged source-native seed at its original dimensions",
        );
        assert!(matches!(
            actions.first(),
            Some(Action::New { tick: 40, .. })
        ));
        assert!(actions.contains(&Action::Draw));
        assert!(actions.contains(&Action::Ticks(10)));
        assert!(actions.contains(&Action::Key("Esc".to_owned())));
        assert!(actions.contains(&Action::Key("Ctrl+G".to_owned())));
        assert!(actions.contains(&Action::Key("g".to_owned())));
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, Action::Resize(s) if s == &[(60, 15), (72, 20)]))
        );
        assert!(actions.contains(&Action::Sweep));
        assert!(
            actions
                .iter()
                .any(|a| matches!(a, Action::Route { test, .. } if test == "x"))
        );
        assert!(actions.iter().any(|a| matches!(a, Action::Raw(_))));
    }

    #[test]
    fn bare_key_recognizes_register_tokens() {
        for key in [
            "Esc",
            "Enter",
            "F10",
            "Alt+1",
            "Ctrl+Shift+Z",
            "Space",
            "Up",
            "/",
            "y",
        ] {
            assert!(is_bare_key(key), "{key} must parse as a key");
        }
        for prose in [
            "Tab until original focus repeats",
            "checkpoint",
            "repeat File and Help",
            "fresh route per row",
        ] {
            assert!(!is_bare_key(prose), "{prose} must stay prose");
        }
    }
}
