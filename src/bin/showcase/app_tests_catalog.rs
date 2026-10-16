//! Showcase coverage gate (plan §7, SC0).
//!
//! 1. The capability registry (`junie_tui::catalog::CAPABILITIES`) matches
//!    the library source: every public type, variant, builder and theme
//!    token is listed, and nothing listed is gone.
//! 2. Every capability is shown by the showcase (`coverage::SHOWN`) or
//!    allowlisted in `tests/showcase_coverage_allow.txt`; allowlist lines
//!    that are stale (now shown, or an unknown id) fail.
//! 3. Every declaration holds in the running showcase.
//!
//! On failure the messages print paste-ready lines. The remaining gaps per
//! SC step are printed by `remaining_gaps_per_step` (run it with
//! `--no-capture`).

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;

use junie_tui::catalog::{CAPABILITIES, CapKind, Capability};
use junie_tui::theme::Theme;
use ratatui::{Terminal, backend::TestBackend};

use crate::app::{App, PageId};
use crate::catalog_scan::{self, Item};
use crate::coverage::{SHOWN, Shown};
use crate::inspector::Tab;
use crate::options::{self, Effect, Opt};
use crate::pages::template::Section;
use junie_tui::ui::specimen::SpecimenState;

const ALLOWLIST: &str = include_str!("../../../tests/showcase_coverage_allow.txt");

fn src_dir() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src"))
}

fn ctor(kind: CapKind) -> &'static str {
    match kind {
        CapKind::Type => "ty",
        CapKind::Variant => "variant",
        CapKind::Builder => "builder",
        CapKind::Token => "token",
        CapKind::State => "state",
    }
}

/// Kinds the source scan owns; `State` entries are hand-listed.
fn scanned_kind(kind: CapKind) -> bool {
    kind != CapKind::State
}

/// Registry vs source: missing entries, entries no longer in the source,
/// kind mismatches and duplicates.
fn registry_problems(caps: &[Capability], scanned: &[Item]) -> Vec<String> {
    let mut problems = Vec::new();
    let mut seen = HashSet::new();
    for cap in caps {
        if !seen.insert(cap.id) {
            problems.push(format!("registry lists {} twice", cap.id));
        }
    }
    let by_id: HashMap<&str, &Capability> = caps.iter().map(|c| (c.id, c)).collect();
    let in_src: HashMap<&str, &Item> = scanned.iter().map(|i| (i.id.as_str(), i)).collect();
    for item in scanned {
        match by_id.get(item.id.as_str()) {
            None => problems.push(format!(
                "missing from src/catalog.rs ({}): Capability::{}(\"{}\"),",
                item.at,
                ctor(item.kind),
                item.id
            )),
            Some(cap) if cap.kind != item.kind => problems.push(format!(
                "{} is a {} in the source but a {} in src/catalog.rs",
                item.id,
                item.kind.name(),
                cap.kind.name()
            )),
            Some(_) => {}
        }
    }
    for cap in caps.iter().filter(|c| scanned_kind(c.kind)) {
        if !in_src.contains_key(cap.id) {
            problems.push(format!(
                "src/catalog.rs lists {} ({}), which the source no longer has: remove it \
                 (and its showcase declaration or allowlist line)",
                cap.id,
                cap.kind.name()
            ));
        }
    }
    problems
}

/// One allowlist line: `id | reason | SCn` or `id | reason | permanent`.
#[derive(Debug)]
struct Allow<'a> {
    id: &'a str,
    step: &'a str,
}

fn parse_allowlist(text: &str) -> (Vec<Allow<'_>>, Vec<String>) {
    let mut out = Vec::new();
    let mut problems = Vec::new();
    for (n, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split('|').map(str::trim).collect();
        let [id, reason, step] = parts[..] else {
            problems.push(format!(
                "allowlist line {}: expected `id | reason | SCn|permanent`: {line}",
                n + 1
            ));
            continue;
        };
        if reason.is_empty() {
            problems.push(format!("allowlist line {}: {id} has no reason", n + 1));
        }
        let valid_step = step == "permanent"
            || step
                .strip_prefix("SC")
                .is_some_and(|d| !d.is_empty() && d.bytes().all(|b| b.is_ascii_digit()));
        if !valid_step {
            problems.push(format!(
                "allowlist line {}: {id} is pending without an SC step (got {step:?}); \
                 use `SCn` or `permanent`",
                n + 1
            ));
        }
        out.push(Allow { id, step });
    }
    (out, problems)
}

/// Coverage: every capability declared or allowlisted, no stale lines,
/// no declaration for an unknown id.
fn coverage_problems(caps: &[Capability], shown: &[(&str, Shown)], allow: &str) -> Vec<String> {
    let (allow, mut problems) = parse_allowlist(allow);
    let known: HashSet<&str> = caps.iter().map(|c| c.id).collect();
    let declared: HashSet<&str> = shown.iter().map(|(id, _)| *id).collect();
    let mut listed = HashSet::new();
    for line in &allow {
        if !listed.insert(line.id) {
            problems.push(format!("allowlist lists {} twice", line.id));
        }
        if let Some(rest) = line.id.strip_prefix("app::") {
            // An app-local pattern deliberately kept out of the library.
            let bin = rest.split("::").next().unwrap_or_default();
            if line.step != "permanent" {
                problems.push(format!(
                    "allowlist line {}: an app-local exclusion must be `permanent`; a pending \
                     app pattern is extracted to the library and gets a catalog id",
                    line.id
                ));
            }
            if bin.is_empty() || !src_dir().join("bin").join(bin).is_dir() {
                problems.push(format!(
                    "stale allowlist line: {} names no app under src/bin; delete it",
                    line.id
                ));
            }
        } else if !known.contains(line.id) {
            problems.push(format!(
                "stale allowlist line: {} is not a capability in src/catalog.rs; delete it",
                line.id
            ));
        } else if declared.contains(line.id) {
            problems.push(format!(
                "stale allowlist line: {} is now shown (src/bin/showcase/coverage.rs); \
                 delete its line ({})",
                line.id, line.step
            ));
        }
    }
    for (id, _) in shown {
        if !known.contains(id) {
            problems.push(format!(
                "coverage.rs declares {id}, which is not a capability in src/catalog.rs"
            ));
        }
    }
    for cap in caps {
        if !declared.contains(cap.id) && !listed.contains(cap.id) {
            problems.push(format!(
                "{} {} is neither shown (src/bin/showcase/coverage.rs) nor allowlisted \
                 (tests/showcase_coverage_allow.txt): `{} | <reason> | SCn`",
                cap.kind.name(),
                cap.id,
                cap.id
            ));
        }
    }
    problems
}

fn fail_on(title: &str, problems: &[String]) {
    assert!(
        problems.is_empty(),
        "{title}: {} problem(s)\n{}",
        problems.len(),
        problems.join("\n")
    );
}

#[test]
fn registry_matches_library_source() {
    let scanned = catalog_scan::scan_tree(src_dir());
    assert!(
        scanned.len() > 100,
        "scan found too little: {}",
        scanned.len()
    );
    fail_on(
        "src/catalog.rs drifted from the library source",
        &registry_problems(CAPABILITIES, &scanned),
    );
}

#[test]
fn every_capability_is_shown_or_allowlisted() {
    fail_on(
        "showcase coverage gate",
        &coverage_problems(CAPABILITIES, SHOWN, ALLOWLIST),
    );
}

/// Collapses whitespace runs to one space, so labels match across the
/// column padding of the rendered page.
fn squash(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The rows of a 220x80 frame of `app`, whitespace runs collapsed.
fn frame_text(app: &mut App) -> String {
    let mut term = Terminal::new(TestBackend::new(220, 80)).unwrap();
    term.draw(|frame| app.render(frame)).unwrap();
    let buf = term.backend().buffer();
    let mut text = String::new();
    for y in 0..buf.area.height {
        let mut row = String::new();
        for x in 0..buf.area.width {
            row.push_str(buf[(x, y)].symbol());
        }
        text.push_str(&squash(&row));
        text.push('\n');
    }
    text
}

/// Text of every rendered row of `page` at a generous size, one row per
/// line, whitespace runs collapsed. A page on the Playground | Options |
/// States template renders each of its sections.
fn page_text(page: PageId) -> String {
    let mut app = App::new(Theme::junie());
    app.goto(page);
    let sections = app.pages[page.index()].template().is_some();
    if !sections {
        return frame_text(&mut app);
    }
    let mut text = String::new();
    for section in Section::ALL {
        if let Some(t) = app.pages[page.index()].template_mut() {
            t.set_section(section);
        }
        text.push_str(&frame_text(&mut app));
    }
    text
}

/// The States rows `page` draws whole in its States section at 220x80,
/// and the frame's text; `None` when the page has no States section.
fn page_states(page: PageId) -> Option<(Vec<SpecimenState>, String)> {
    let mut app = App::new(Theme::junie());
    app.goto(page);
    app.pages[page.index()]
        .template_mut()?
        .set_section(Section::States);
    let text = frame_text(&mut app);
    let rows = app.pages[page.index()].template()?.state_rows().to_vec();
    Some((rows, text))
}

/// Every cell of a 220x80 frame of `app`, symbol and style.
fn frame_cells(app: &mut App) -> Vec<ratatui::buffer::Cell> {
    let mut term = Terminal::new(TestBackend::new(220, 80)).unwrap();
    term.draw(|frame| app.render(frame)).unwrap();
    term.backend().buffer().content().to_vec()
}

/// What option value `value` shows on `page`: the page's frame, every
/// section of a templated page, with the Inspector closed; for a
/// behaviour option, the live configuration and the Inspector's Options
/// tab instead. `needs` are set first, each through `--opt`.
fn option_render(page: PageId, opt: &Opt, value: usize) -> (Vec<ratatui::buffer::Cell>, String) {
    let mut app = App::new(Theme::junie());
    app.goto(page);
    let set = |app: &mut App, key: &str, value: &str| {
        let arg = format!("{key}={value}");
        app.apply_opt(options::parse_opt(&arg).unwrap_or_else(|e| panic!("{arg}: {e}")));
    };
    for (k, v) in opt.needs {
        set(&mut app, k, v);
    }
    set(&mut app, opt.key, opt.values[value]);
    let mut cells = Vec::new();
    match opt.effect {
        Effect::Frame if app.pages[page.index()].template().is_some() => {
            for section in Section::ALL {
                if let Some(t) = app.pages[page.index()].template_mut() {
                    t.set_section(section);
                }
                cells.extend(frame_cells(&mut app));
            }
        }
        Effect::Frame => cells = frame_cells(&mut app),
        Effect::Behaviour => {
            // a first frame sizes the app: at 220 columns the pane docks
            frame_cells(&mut app);
            app.toggle_inspector();
            app.insp.set_tab(Tab::Options);
            cells = frame_cells(&mut app);
        }
    }
    (cells, app.live_config())
}

/// `Option(slug, key)` holds when every value it names, set through the
/// `--opt` path, shows differently from another value of the option.
/// `key` is `shell.color` (every value) or `shell.color=mono` (one).
fn option_holds(slug: &str, key: &str) -> Result<(), String> {
    let page = PageId::from_name(slug).ok_or_else(|| format!("unknown page {slug:?}"))?;
    let (name, only) = match key.split_once('=') {
        Some((k, v)) => (k, Some(v)),
        None => (key, None),
    };
    let opt = options::find(name).ok_or_else(|| format!("no option {name:?}"))?;
    if let Some(owner) = options::page_of(opt.key)
        && owner != page
    {
        return Err(format!("{} is an option of another page", opt.key));
    }
    let values: Vec<usize> = match only {
        Some(v) => vec![
            opt.index_of(v)
                .ok_or_else(|| format!("{v:?} is not a value of {}", opt.key))?,
        ],
        None => (0..opt.values.len()).collect(),
    };
    option_values_hold(page, opt, &values)
}

/// Each of `values` renders unlike another value of `opt` (the default, or
/// for the default itself the next value).
fn option_values_hold(page: PageId, opt: &Opt, values: &[usize]) -> Result<(), String> {
    for &v in values {
        let other = if v == opt.default {
            (opt.default + 1) % opt.values.len()
        } else {
            opt.default
        };
        let (cells, config) = option_render(page, opt, v);
        let (base_cells, base_config) = option_render(page, opt, other);
        let shows = match opt.effect {
            Effect::Frame => cells != base_cells,
            Effect::Behaviour => config != base_config && cells != base_cells,
        };
        if !shows {
            return Err(format!(
                "{}={} shows the same as {}={} ({:?})",
                opt.key, opt.values[v], opt.key, opt.values[other], opt.effect
            ));
        }
    }
    Ok(())
}

/// Problems with the declarations themselves: each must hold in the
/// running showcase.
fn declaration_problems(shown: &[(&str, Shown)]) -> Vec<String> {
    let mut problems = Vec::new();
    let mut pages: HashMap<&str, String> = HashMap::new();
    let mut states: HashMap<&str, Option<(Vec<SpecimenState>, String)>> = HashMap::new();
    for (id, how) in shown {
        match *how {
            Shown::Demo(slug, label) => {
                let Some(page) = PageId::from_name(slug) else {
                    problems.push(format!("{id}: Demo on unknown page {slug:?}"));
                    continue;
                };
                let text = pages.entry(slug).or_insert_with(|| page_text(page));
                if !text.contains(&squash(label)) {
                    problems.push(format!(
                        "{id}: page {slug:?} does not render {label:?} (Demo declaration)"
                    ));
                }
            }
            // set through the `--opt` path, the value must change what
            // the page shows (or, for a behaviour option, the live
            // configuration and the Options tab)
            Shown::Option(slug, key) => {
                if let Err(e) = option_holds(slug, key) {
                    problems.push(format!("{id}: Option({slug:?}, {key:?}): {e}"));
                }
            }
            // the page's States section draws the row whole, under its label
            Shown::StateRow(slug, row) => {
                let Some(page) = PageId::from_name(slug) else {
                    problems.push(format!("{id}: StateRow on unknown page {slug:?}"));
                    continue;
                };
                let Some(state) = SpecimenState::from_slug(row) else {
                    problems.push(format!("{id}: {row:?} is not a States row name"));
                    continue;
                };
                let grid = states.entry(slug).or_insert_with(|| page_states(page));
                match grid {
                    None => problems.push(format!(
                        "{id}: page {slug:?} has no States section (StateRow declaration)"
                    )),
                    Some((rows, text)) => {
                        let labelled = text.lines().any(|l| l.contains(state.label()));
                        if !rows.contains(&state) || !labelled {
                            problems.push(format!(
                                "{id}: page {slug:?} draws no States row {row:?} (StateRow declaration)"
                            ));
                        }
                    }
                }
            }
        }
    }
    problems
}

#[test]
fn showcase_declarations_hold() {
    fail_on(
        "src/bin/showcase/coverage.rs declares what the showcase does not show",
        &declaration_problems(SHOWN),
    );
}

#[test]
fn remaining_gaps_per_step() {
    let (allow, _) = parse_allowlist(ALLOWLIST);
    let mut per_step: BTreeMap<(u32, &str), usize> = BTreeMap::new();
    for line in &allow {
        let n = line
            .step
            .strip_prefix("SC")
            .and_then(|d| d.parse().ok())
            .unwrap_or(u32::MAX);
        *per_step.entry((n, line.step)).or_default() += 1;
    }
    println!(
        "showcase coverage: {} capabilities, {} shown, {} allowlisted",
        CAPABILITIES.len(),
        SHOWN.len(),
        allow.len()
    );
    for ((_, step), count) in &per_step {
        println!("  {step:>9}: {count}");
    }
}

// ------------------------------------------------- the gate proves itself

/// Fixture: a new public widget with a new builder, scanned as if it were
/// `src/widgets/fixture.rs`. Nothing in the real source changes.
const NEW_BUILDER: &str = r#"
    pub struct Gizmo { glow: bool }
    impl Gizmo {
        pub fn new() -> Self { Self { glow: false } }
        pub fn glow(mut self, on: bool) -> Self { self.glow = on; self }
    }
"#;

#[test]
fn gate_fails_when_a_new_builder_has_no_coverage() {
    let mut scanned = catalog_scan::scan_tree(src_dir());
    scanned.extend(catalog_scan::scan_source(
        &["widgets", "fixture"],
        NEW_BUILDER,
    ));

    // Not in the registry: the drift check names both new items.
    let drift = registry_problems(CAPABILITIES, &scanned);
    assert!(
        drift
            .iter()
            .any(|p| p.contains("Capability::builder(\"widgets::fixture::Gizmo::glow\")")),
        "{drift:#?}"
    );
    assert!(
        drift
            .iter()
            .any(|p| p.contains("Capability::ty(\"widgets::fixture::Gizmo\")")),
        "{drift:#?}"
    );

    // Registered but neither shown nor allowlisted: the coverage check
    // fails on it.
    let mut caps = CAPABILITIES.to_vec();
    caps.push(Capability::ty("widgets::fixture::Gizmo"));
    caps.push(Capability::builder("widgets::fixture::Gizmo::glow"));
    assert!(registry_problems(&caps, &scanned).is_empty());
    let gaps = coverage_problems(&caps, SHOWN, ALLOWLIST);
    assert!(
        gaps.iter()
            .any(|p| p.contains("builder widgets::fixture::Gizmo::glow is neither shown")),
        "{gaps:#?}"
    );

    // An allowlist line against its SC step satisfies the gate.
    let allow = format!(
        "{ALLOWLIST}\nwidgets::fixture::Gizmo | fixture | SC7\n\
         widgets::fixture::Gizmo::glow | fixture | SC7\n"
    );
    assert_eq!(
        coverage_problems(&caps, SHOWN, &allow),
        Vec::<String>::new()
    );
}

#[test]
fn gate_fails_on_a_stale_allowlist_line() {
    let (shown_id, _) = SHOWN.first().expect("at least one declaration");

    // Allowlisted although the showcase now shows it.
    let allow = format!("{ALLOWLIST}\n{shown_id} | was pending | SC2\n");
    let problems = coverage_problems(CAPABILITIES, SHOWN, &allow);
    assert!(
        problems
            .iter()
            .any(|p| p.contains(&format!("stale allowlist line: {shown_id} is now shown"))),
        "{problems:#?}"
    );

    // An id the registry does not know.
    let allow = format!("{ALLOWLIST}\nwidgets::gone::Gone | removed | SC7\n");
    let problems = coverage_problems(CAPABILITIES, SHOWN, &allow);
    assert!(
        problems
            .iter()
            .any(|p| p.contains("stale allowlist line: widgets::gone::Gone")),
        "{problems:#?}"
    );

    // A pending line without its SC step.
    let allow = format!("{ALLOWLIST}\nwidgets::gone::Gone | todo | later\n");
    let problems = coverage_problems(CAPABILITIES, SHOWN, &allow);
    assert!(
        problems
            .iter()
            .any(|p| p.contains("pending without an SC step")),
        "{problems:#?}"
    );

    // An app-local exclusion for an app that no longer exists, and one
    // that is pending instead of permanent.
    let allow = format!(
        "{ALLOWLIST}\napp::gone_app::intro | brand | permanent\n\
         app::jackin_preview::banner | brand | SC4\n"
    );
    let problems = coverage_problems(CAPABILITIES, SHOWN, &allow);
    assert!(
        problems
            .iter()
            .any(|p| p.contains("stale allowlist line: app::gone_app::intro")),
        "{problems:#?}"
    );
    assert!(
        problems
            .iter()
            .any(|p| p.contains("app::jackin_preview::banner: an app-local exclusion must be")),
        "{problems:#?}"
    );
}

#[test]
fn gate_fails_when_a_declaration_is_removed() {
    let (removed, _) = SHOWN.first().expect("at least one declaration");
    let fewer: Vec<(&str, Shown)> = SHOWN
        .iter()
        .filter(|(id, _)| id != removed)
        .copied()
        .collect();
    let problems = coverage_problems(CAPABILITIES, &fewer, ALLOWLIST);
    assert!(
        problems
            .iter()
            .any(|p| p.contains(&format!("{removed} is neither shown"))),
        "{problems:#?}"
    );
}

#[test]
fn gate_sets_an_option_through_opt_and_needs_a_visible_change() {
    // a real declaration: mono redraws the page; Release (the showcase's
    // default focus policy) differs from Press in the live configuration
    assert_eq!(option_holds("overview", "shell.color=mono"), Ok(()));
    assert_eq!(option_holds("overview", "shell.focus_on=release"), Ok(()));
    // fixture: the level overrides without the level they override change
    // nothing, so a declaration of them would not hold
    let real = options::find("shell.overrides").unwrap();
    let inert = Opt {
        needs: &[],
        ..*real
    };
    let err = option_values_hold(PageId::Overview, &inert, &[1]).unwrap_err();
    assert!(err.contains("shows the same"), "{err}");
    // with what it needs, the same value holds
    assert_eq!(option_values_hold(PageId::Overview, real, &[1]), Ok(()));
    // unknown keys, values and pages
    for (slug, key) in [
        ("overview", "shell.glow"),
        ("overview", "shell.color=purple"),
        ("nosuchpage", "shell.color"),
    ] {
        assert!(option_holds(slug, key).is_err(), "{slug} {key}");
    }
}

#[test]
fn gate_fails_on_a_declaration_the_showcase_does_not_hold() {
    let (id, _) = SHOWN.first().expect("at least one declaration");
    let wrong = [
        (*id, Shown::Demo("buttons", "no such text anywhere")),
        (*id, Shown::Demo("nosuchpage", "x")),
        (*id, Shown::Option("buttons", "glow")),
        // a row the page's grid does not have, a row name that is no
        // state, a page with no States section, an unknown page
        (*id, Shown::StateRow("buttons", "error")),
        (*id, Shown::StateRow("inputs", "sideways")),
        (*id, Shown::StateRow("diff", "rest")),
        (*id, Shown::StateRow("nosuchpage", "rest")),
    ];
    assert_eq!(declaration_problems(&wrong).len(), wrong.len());
}
