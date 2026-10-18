















use junie_tui::runtime::Application;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;

use junie_tui::catalog::{CAPABILITIES, CapKind, Capability};
use junie_tui::theme::Theme;
use ratatui::{Terminal, backend::TestBackend};

use crate::app::{App, PageId};
use crate::catalog_scan::{self, Item};
use crate::coverage::{self, SHOWN, Shown, ViaRole};
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
        CapKind::Function => "function",
        CapKind::State => "state",
    }
}


fn scanned_kind(kind: CapKind) -> bool {
    kind != CapKind::State
}



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
    problems.extend(via_problems(caps, shown));
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





fn via_problems(caps: &[Capability], shown: &[(&str, Shown)]) -> Vec<String> {
    let kinds: HashMap<&str, CapKind> = caps.iter().map(|c| (c.id, c.kind)).collect();
    let direct: HashSet<&str> = shown
        .iter()
        .filter(|(_, how)| !matches!(how, Shown::Via(_)))
        .map(|(id, _)| *id)
        .collect();
    let mut problems = Vec::new();
    let mut seen = HashSet::new();
    for (id, how) in shown {
        let Shown::Via(owner) = *how else { continue };
        if !seen.insert(*id) {
            problems.push(format!("coverage.rs VIA lists {id} twice"));
        }
        if direct.contains(id) {
            problems.push(format!(
                "stale Via: {id} is declared directly in SHOWN; delete it from VIA"
            ));
        }
        match kinds.get(id) {
            Some(CapKind::Function) | None => {}
            Some(kind) => problems.push(format!(
                "{id}: Via covers functions only; it is a {}",
                kind.name()
            )),
        }
        let parent = id.rsplit_once("::").map_or("", |(p, _)| p);
        if kinds.get(parent) == Some(&CapKind::Type) && parent != owner {
            problems.push(format!(
                "{id}: Via({owner}): a method is covered only by the demo of its own type {parent}"
            ));
        }
        if kinds.get(owner) != Some(&CapKind::Type) {
            problems.push(format!(
                "{id}: Via({owner}): {owner} is not a type in src/catalog.rs"
            ));
        } else if !direct.contains(owner) {
            problems.push(format!(
                "{id}: Via({owner}): {owner} is not demonstrated (no Demo, Option or StateRow \
                 in SHOWN); show it, or allowlist {id} against the step that shows {owner}"
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
        &coverage_problems(CAPABILITIES, coverage::declarations(), ALLOWLIST),
    );
}



fn squash(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}


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


fn frame_cells(app: &mut App) -> Vec<ratatui::buffer::Cell> {
    let mut term = Terminal::new(TestBackend::new(220, 80)).unwrap();
    term.draw(|frame| app.render(frame)).unwrap();
    term.backend().buffer().content().to_vec()
}





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

            frame_cells(&mut app);
            app.toggle_inspector();
            app.insp.set_tab(Tab::Options);
            cells = frame_cells(&mut app);
        }
    }
    (cells, app.live_config())
}




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



            Shown::Option(slug, key) => {
                if let Err(e) = option_holds(slug, key) {
                    problems.push(format!("{id}: Option({slug:?}, {key:?}): {e}"));
                }
            }

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


            Shown::Via(_) => {}
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
    let via = coverage::declarations().len() - SHOWN.len();
    println!(
        "showcase coverage: {} capabilities, {} shown directly, {via} via their owner, {} allowlisted",
        CAPABILITIES.len(),
        SHOWN.len(),
        allow.len()
    );
    for ((_, step), count) in &per_step {
        println!("  {step:>9}: {count}");
    }
    for role in [ViaRole::Protocol, ViaRole::Operation, ViaRole::Plumbing] {
        let count: usize = coverage::VIA
            .iter()
            .filter(|(_, r, _)| *r == role)
            .map(|(_, _, names)| names.len())
            .sum();
        println!("  via {role:?}: {count} ({})", role.reason());
    }
}





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



    let mut caps = CAPABILITIES.to_vec();
    caps.push(Capability::ty("widgets::fixture::Gizmo"));
    caps.push(Capability::builder("widgets::fixture::Gizmo::glow"));
    assert!(registry_problems(&caps, &scanned).is_empty());
    let gaps = coverage_problems(&caps, coverage::declarations(), ALLOWLIST);
    assert!(
        gaps.iter()
            .any(|p| p.contains("builder widgets::fixture::Gizmo::glow is neither shown")),
        "{gaps:#?}"
    );


    let allow = format!(
        "{ALLOWLIST}\nwidgets::fixture::Gizmo | fixture | SC7\n\
         widgets::fixture::Gizmo::glow | fixture | SC7\n"
    );
    assert_eq!(
        coverage_problems(&caps, coverage::declarations(), &allow),
        Vec::<String>::new()
    );
}




const NEW_FUNCTIONS: &str = r#"
    pub struct Gizmo { glow: u8 }
    impl Gizmo {
        pub fn new() -> Self { Self { glow: 0 } }
        pub fn level(&self) -> u8 { self.glow }
    }
    pub fn render_gizmo(g: &Gizmo) -> String { String::new() }
"#;

#[test]
fn gate_fails_when_a_new_public_function_has_no_coverage() {
    let mut scanned = catalog_scan::scan_tree(src_dir());
    scanned.extend(catalog_scan::scan_source(
        &["widgets", "fixture"],
        NEW_FUNCTIONS,
    ));



    let drift = registry_problems(CAPABILITIES, &scanned);
    for line in [
        "Capability::function(\"widgets::fixture::render_gizmo\")",
        "Capability::function(\"widgets::fixture::Gizmo::level\")",
    ] {
        assert!(drift.iter().any(|p| p.contains(line)), "{line}: {drift:#?}");
    }
    assert!(
        !drift.iter().any(|p| p.contains("Gizmo::new")),
        "{drift:#?}"
    );



    let mut caps = CAPABILITIES.to_vec();
    caps.push(Capability::ty("widgets::fixture::Gizmo"));
    caps.push(Capability::function("widgets::fixture::Gizmo::level"));
    caps.push(Capability::function("widgets::fixture::render_gizmo"));
    assert!(registry_problems(&caps, &scanned).is_empty());
    let gaps = coverage_problems(&caps, coverage::declarations(), ALLOWLIST);
    for id in [
        "widgets::fixture::Gizmo::level",
        "widgets::fixture::render_gizmo",
    ] {
        assert!(
            gaps.iter()
                .any(|p| p.contains(&format!("function {id} is neither shown"))),
            "{id}: {gaps:#?}"
        );
    }


    let gone: Vec<Item> = scanned
        .iter()
        .filter(|i| i.id != "widgets::fixture::render_gizmo")
        .cloned()
        .collect();
    assert!(
        registry_problems(&caps, &gone)
            .iter()
            .any(|p| p.contains("lists widgets::fixture::render_gizmo (function)")),
    );


    let allow = format!(
        "{ALLOWLIST}\nwidgets::fixture::Gizmo | fixture | SC58\n\
         widgets::fixture::Gizmo::level | fixture | SC58\n\
         widgets::fixture::render_gizmo | fixture | SC58\n"
    );
    assert_eq!(
        coverage_problems(&caps, coverage::declarations(), &allow),
        Vec::<String>::new()
    );
}

#[test]
fn gate_fails_on_a_stale_allowlist_line() {
    let (shown_id, _) = SHOWN.first().expect("at least one declaration");


    let allow = format!("{ALLOWLIST}\n{shown_id} | was pending | SC2\n");
    let problems = coverage_problems(CAPABILITIES, coverage::declarations(), &allow);
    assert!(
        problems
            .iter()
            .any(|p| p.contains(&format!("stale allowlist line: {shown_id} is now shown"))),
        "{problems:#?}"
    );


    let allow = format!("{ALLOWLIST}\nwidgets::gone::Gone | removed | SC7\n");
    let problems = coverage_problems(CAPABILITIES, coverage::declarations(), &allow);
    assert!(
        problems
            .iter()
            .any(|p| p.contains("stale allowlist line: widgets::gone::Gone")),
        "{problems:#?}"
    );


    let allow = format!("{ALLOWLIST}\nwidgets::gone::Gone | todo | later\n");
    let problems = coverage_problems(CAPABILITIES, coverage::declarations(), &allow);
    assert!(
        problems
            .iter()
            .any(|p| p.contains("pending without an SC step")),
        "{problems:#?}"
    );



    let allow = format!(
        "{ALLOWLIST}\napp::gone_app::intro | brand | permanent\n\
         app::jackin_preview::banner | brand | SC4\n"
    );
    let problems = coverage_problems(CAPABILITIES, coverage::declarations(), &allow);
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
    let fewer: Vec<(&str, Shown)> = coverage::declarations()
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


    assert_eq!(option_holds("overview", "shell.color=mono"), Ok(()));
    assert_eq!(option_holds("overview", "shell.focus_on=release"), Ok(()));


    let real = options::find("shell.overrides").unwrap();
    let inert = Opt {
        needs: &[],
        ..*real
    };
    let err = option_values_hold(PageId::Overview, &inert, &[1]).unwrap_err();
    assert!(err.contains("shows the same"), "{err}");

    assert_eq!(option_values_hold(PageId::Overview, real, &[1]), Ok(()));

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


        (*id, Shown::StateRow("buttons", "error")),
        (*id, Shown::StateRow("inputs", "sideways")),
        (*id, Shown::StateRow("meters", "rest")),
        (*id, Shown::StateRow("nosuchpage", "rest")),
    ];
    assert_eq!(declaration_problems(&wrong).len(), wrong.len());
}



const GIZMO: &str = "widgets::fixture::Gizmo";
const GIZMO_VIA: &[(&str, ViaRole, &[&str])] = &[(
    GIZMO,
    ViaRole::Protocol,
    &["render", "widgets::fixture::render_gizmo"],
)];

fn gizmo_caps() -> Vec<Capability> {
    let mut caps = CAPABILITIES.to_vec();
    caps.push(Capability::ty(GIZMO));
    caps.push(Capability::function("widgets::fixture::Gizmo::render"));
    caps.push(Capability::function("widgets::fixture::render_gizmo"));
    caps
}


fn with_gizmo(extra: &[(&'static str, Shown)]) -> Vec<(&'static str, Shown)> {
    let mut shown = coverage::declarations().to_vec();
    shown.extend(coverage::expand(extra, GIZMO_VIA));
    shown
}

const GIZMO_DEMO: (&str, Shown) = (GIZMO, Shown::Demo("buttons", "Gizmo"));

#[test]
fn via_holds_when_the_owner_is_demonstrated() {
    let caps = gizmo_caps();
    assert_eq!(
        coverage_problems(&caps, &with_gizmo(&[GIZMO_DEMO]), ALLOWLIST),
        Vec::<String>::new()
    );
}

#[test]
fn via_fails_when_the_owner_is_not_demonstrated() {
    let caps = gizmo_caps();

    for step in ["SC58", "permanent"] {
        let allow = format!("{ALLOWLIST}\n{GIZMO} | fixture | {step}\n");
        let problems = coverage_problems(&caps, &with_gizmo(&[]), &allow);
        for id in [
            "widgets::fixture::Gizmo::render",
            "widgets::fixture::render_gizmo",
        ] {
            assert!(
                problems.iter().any(
                    |p| p.contains(&format!("{id}: Via({GIZMO}): {GIZMO} is not demonstrated"))
                ),
                "{step} {id}: {problems:#?}"
            );
        }
    }

    let (owner, _, names) = coverage::VIA.first().expect("at least one Via group");
    let fewer: Vec<(&str, Shown)> = coverage::declarations()
        .iter()
        .filter(|(id, how)| id != owner || matches!(how, Shown::Via(_)))
        .copied()
        .collect();
    let allow = format!("{ALLOWLIST}\n{owner} | fixture | SC58\n");
    let problems = coverage_problems(CAPABILITIES, &fewer, &allow);
    assert_eq!(
        problems
            .iter()
            .filter(|p| p.contains(&format!("Via({owner}): {owner} is not demonstrated")))
            .count(),
        names.len(),
        "{problems:#?}"
    );
}

#[test]
fn a_stale_via_fails() {
    let shown = with_gizmo(&[GIZMO_DEMO]);


    let gone: Vec<Capability> = gizmo_caps()
        .into_iter()
        .filter(|c| c.id != "widgets::fixture::render_gizmo")
        .collect();
    let problems = coverage_problems(&gone, &shown, ALLOWLIST);
    assert!(
        problems.iter().any(|p| p.contains(
            "coverage.rs declares widgets::fixture::render_gizmo, which is not a capability"
        )),
        "{problems:#?}"
    );

    let caps = gizmo_caps();

    let mut both = shown.clone();
    both.push((
        "widgets::fixture::Gizmo::render",
        Shown::Demo("buttons", "Gizmo"),
    ));
    let problems = coverage_problems(&caps, &both, ALLOWLIST);
    assert!(
        problems
            .iter()
            .any(|p| p.contains("stale Via: widgets::fixture::Gizmo::render is declared directly")),
        "{problems:#?}"
    );


    let allow = format!("{ALLOWLIST}\nwidgets::fixture::render_gizmo | fixture | SC58\n");
    let problems = coverage_problems(&caps, &shown, &allow);
    assert!(
        problems.iter().any(
            |p| p.contains("stale allowlist line: widgets::fixture::render_gizmo is now shown")
        ),
        "{problems:#?}"
    );



    let theme = "theme::Theme";
    let mut wrong = shown.clone();
    wrong.retain(|(id, _)| *id != "widgets::fixture::Gizmo::render");
    wrong.push(("widgets::fixture::Gizmo::render", Shown::Via(theme)));
    wrong.push(("widgets::empty::EmptyState", Shown::Via(GIZMO)));
    let problems = coverage_problems(&caps, &wrong, ALLOWLIST);
    for needle in [
        "widgets::fixture::Gizmo::render: Via(theme::Theme): a method is covered only by the \
         demo of its own type widgets::fixture::Gizmo",
        "widgets::empty::EmptyState: Via covers functions only; it is a type",
    ] {
        assert!(
            problems.iter().any(|p| p.contains(needle)),
            "{needle}: {problems:#?}"
        );
    }
}
