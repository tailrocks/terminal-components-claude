//! Bounded source inventory and behavioral examples for the showcase.
//!
//! The gate checks three separate contracts:
//!
//! 1. the declared scanner syntax scope matches the checked-in inventory;
//! 2. every inventory ID has a bounded behavior example or explicit review
//!    disposition;
//! 3. historical page/title leads remain live navigation hints and are never
//!    treated as behavior proof.

#[path = "coverage/mod.rs"]
mod coverage;

use std::collections::{BTreeMap, HashMap, HashSet};

use crate::coverage::catalog::{CAPABILITIES, CapKind, Capability};
use crate::coverage::declarations::{
    self, ALLOWLIST, Allow, BehaviorProof, PAGE_LEADS, PROOFS, PageLead, ProofKind,
};
use crate::coverage::scan::{self, Item};
use showcase_ui::{App, NAV_ENTRIES, PageId};
use termrock::{Id, KeyCode, Part, PartRef, StateFlags, Theme};
use termrock_test_support::Harness;

const WIDTH: u16 = 120;
const HEIGHT: u16 = 40;
const DIFF_TABS: Id = Id::root("showcase_ui::pages::diff.page_tabs");
const DIFF_MODE: Id = Id::root("showcase_ui::pages::diff.option.mode");
const DIFF_VIEW: Id = Id::root("showcase_ui::pages::diff.view");
const TEXTAREAS_PAGE_TABS: Id = Id::root("showcase_ui::pages::textareas::page_tabs");
const TEXTAREAS_BODY: Id = Id::root("showcase_ui::pages::textareas::textareas.body");
const TEXTAREAS_BLUR_ACTION: Id = Id::root("showcase_ui::pages::textareas::option.blur_action");
const TEXTAREAS_EDIT_ENTRY: Id = Id::root("showcase_ui::pages::textareas::option.edit_entry");
const TEXTAREAS_FOCUS_SELECT: Id = Id::root("showcase_ui::pages::textareas::option.focus_select");
const TEXTAREAS_ESCAPE_BEHAVIOR: Id =
    Id::root("showcase_ui::pages::textareas::option.escape_behavior");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProofControl {
    DiffDemo,
    ReviewVariant,
    SetMode,
    StateRow,
    SharedDraw,
    SharedUpdate,
    TextAreaEditEntry,
    TextAreaFocusSelect,
    TextAreaEscReverts,
}

// This closed table is the only source of proof credit. The order matches
// ProofControl::index. Each route test exercises its own listed controls.
const EXECUTABLE_PROOFS: [BehaviorProof; 9] = [
    BehaviorProof {
        capability: "termrock-editors::diff::DiffView",
        page: "diff",
        route: "Diff",
        kind: ProofKind::Demo,
        probe: "config/service.toml",
        owner: None,
    },
    BehaviorProof {
        capability: "termrock-editors::diff::DiffMode::Review",
        page: "diff",
        route: "Diff",
        kind: ProofKind::Option,
        probe: " · review",
        owner: None,
    },
    BehaviorProof {
        capability: "termrock-editors::diff::DiffViewState::set_mode",
        page: "diff",
        route: "Diff",
        kind: ProofKind::Option,
        probe: " · review",
        owner: None,
    },
    BehaviorProof {
        capability: "termrock-editors::diff::DiffViewState",
        page: "diff",
        route: "Diff",
        kind: ProofKind::StateRow,
        probe: "No file selected",
        owner: None,
    },
    BehaviorProof {
        capability: "termrock-editors::diff::DiffView::draw",
        page: "diff",
        route: "Diff",
        kind: ProofKind::Via,
        probe: "config/service.toml",
        owner: Some("termrock-editors::diff::DiffView"),
    },
    BehaviorProof {
        capability: "termrock-editors::diff::DiffView::update",
        page: "diff",
        route: "Diff",
        kind: ProofKind::Via,
        probe: " · review",
        owner: Some("termrock-editors::diff::DiffView"),
    },
    BehaviorProof {
        capability: "termrock-fields::textarea::TextArea::edit_entry",
        page: "textareas",
        route: "Text areas",
        kind: ProofKind::Option,
        probe: "Type to edit",
        owner: None,
    },
    BehaviorProof {
        capability: "termrock-fields::textarea::TextArea::focus_select",
        page: "textareas",
        route: "Text areas",
        kind: ProofKind::Option,
        probe: "Select all",
        owner: None,
    },
    BehaviorProof {
        capability: "termrock-fields::textarea::TextArea::esc_reverts",
        page: "textareas",
        route: "Text areas",
        kind: ProofKind::Option,
        probe: "Revert",
        owner: None,
    },
];

impl ProofControl {
    const fn index(self) -> usize {
        match self {
            Self::DiffDemo => 0,
            Self::ReviewVariant => 1,
            Self::SetMode => 2,
            Self::StateRow => 3,
            Self::SharedDraw => 4,
            Self::SharedUpdate => 5,
            Self::TextAreaEditEntry => 6,
            Self::TextAreaFocusSelect => 7,
            Self::TextAreaEscReverts => 8,
        }
    }
}

fn executable_proof(control: ProofControl) -> &'static BehaviorProof {
    &EXECUTABLE_PROOFS[control.index()]
}

fn diff_harness() -> Harness<App> {
    let route = executable_proof(ProofControl::DiffDemo).route;
    Harness::new(App::for_page_name(route), Theme::junie(), WIDTH, HEIGHT)
}

fn textareas_harness() -> Harness<App> {
    let route = executable_proof(ProofControl::TextAreaEditEntry).route;
    let harness = Harness::new(App::for_page_name(route), Theme::junie(), WIDTH, HEIGHT);
    assert_eq!(harness.app().page(), PageId::TextAreas);
    harness
}

fn assert_control_proof(control: ProofControl, page: &str, route: &str) {
    let proof = executable_proof(control);
    assert_eq!(proof.page, page, "{} proof page", proof.capability);
    assert_eq!(proof.route, route, "{} proof route", proof.capability);
    assert!(
        PROOFS.contains(proof),
        "{} behavior row is absent from declarations_data.rs",
        proof.capability
    );
}

#[expect(
    clippy::expect_used,
    reason = "route proofs require the body text part to be visible"
)]
fn textarea_text(harness: &Harness<App>) -> String {
    let area = harness
        .area_of_part(TEXTAREAS_BODY, PartRef::of(Part::TEXT))
        .expect("the Text areas body exposes its visible text part");
    let rows = (area.y..area.bottom())
        .map(|y| {
            let mut row = String::new();
            let mut x = area.x;
            while x < area.right() {
                let symbol = harness.cell(x, y).symbol();
                row.push_str(symbol);
                x = x.saturating_add(termrock::width(symbol).max(1));
            }
            row.trim_end().to_owned()
        })
        .collect::<Vec<_>>();
    rows.join("\n").trim_end_matches('\n').to_owned()
}

fn seed_textarea(harness: &mut Harness<App>, value: &str) {
    assert!(
        harness.tab_to(TEXTAREAS_BODY),
        "the Text areas body is reachable"
    );
    assert!(
        harness
            .state_of(TEXTAREAS_BODY)
            .contains(StateFlags::EDITING),
        "the default Immediate body begins an edit session"
    );
    let _ = harness.ctrl('l');
    let _ = harness.paste(value);
    let _ = harness.key(KeyCode::Esc);
    assert!(
        !harness
            .state_of(TEXTAREAS_BODY)
            .contains(StateFlags::EDITING),
        "the seeded value is committed"
    );
    assert_eq!(textarea_text(harness), value, "the short seed is exact");
}

fn choose_textareas_option(
    harness: &mut Harness<App>,
    option: Id,
    steps: &[KeyCode],
    selected_label: &str,
) {
    assert!(
        harness.tab_to(TEXTAREAS_PAGE_TABS),
        "Text areas section tabs focus"
    );
    let _ = harness.key(KeyCode::Char('2'));
    assert!(
        harness.area_of(option).is_some(),
        "option {option:?} is visible"
    );
    assert!(harness.tab_to(option), "option {option:?} receives focus");
    let _ = harness.key(KeyCode::Enter);
    assert!(harness.is_open(option), "option {option:?} opens natively");
    for key in steps {
        let _ = harness.key(*key);
    }
    let _ = harness.key(KeyCode::Enter);
    assert!(
        !harness.is_open(option),
        "option {option:?} closes after choice"
    );
    let option_text = harness.text();
    assert!(
        option_text.contains(selected_label),
        "option {option:?} shows {selected_label:?}\n{option_text}"
    );
    assert!(
        harness.tab_to(TEXTAREAS_PAGE_TABS),
        "section tabs regain focus"
    );
    let _ = harness.key(KeyCode::Char('1'));
    assert!(
        harness.area_of(option).is_none(),
        "option {option:?} is hidden"
    );
}

/// The page with this slug.
fn page_by_slug(slug: &str) -> Option<PageId> {
    PageId::ALL.iter().copied().find(|page| page.slug() == slug)
}

/// The problems with the catalogue against a fresh scan of the library.
fn registry_problems(caps: &[Capability], scanned: &[Item]) -> Vec<String> {
    let mut problems = Vec::new();
    let mut seen = HashSet::new();
    for cap in caps {
        if !seen.insert(cap.id) {
            problems.push(format!("the catalogue lists {} twice", cap.id));
        }
    }
    let by_id: HashMap<&str, &Capability> = caps.iter().map(|c| (c.id, c)).collect();
    let in_src: HashMap<&str, &Item> = scanned.iter().map(|i| (i.id.as_str(), i)).collect();
    for item in scanned {
        match by_id.get(item.id.as_str()) {
            None => problems.push(format!(
                "missing from the catalogue ({}): Capability::{}(\"{}\"),",
                item.at,
                item.kind.ctor(),
                item.id
            )),
            Some(cap) if cap.kind != item.kind => problems.push(format!(
                "{} is a {} in the source but a {} in the catalogue",
                item.id,
                item.kind.name(),
                cap.kind.name()
            )),
            Some(_) => {}
        }
    }
    for cap in caps {
        if !in_src.contains_key(cap.id) {
            problems.push(format!(
                "the catalogue lists {} ({}), which the source no longer has: remove it \
                 (and its showcase declaration or allowlist line)",
                cap.id,
                cap.kind.name()
            ));
        }
    }
    problems
}

/// The problems with explicit behavioral claims and pending dispositions.
fn coverage_problems(
    caps: &[Capability],
    proofs: &[BehaviorProof],
    allow_text: &str,
) -> Vec<String> {
    let (allow, mut problems) = declarations::parse_allowlist(allow_text);
    let known: HashSet<&str> = caps.iter().map(|cap| cap.id).collect();
    let proof_ids: HashSet<&str> = proofs.iter().map(|proof| proof.capability).collect();
    let mut listed = HashSet::new();
    for line in &allow {
        if !listed.insert(line.id) {
            problems.push(format!("the disposition file lists {} twice", line.id));
        }
        if !known.contains(line.id) {
            problems.push(format!(
                "stale disposition: {} is not in the current inventory",
                line.id
            ));
        }
        if proof_ids.contains(line.id) {
            problems.push(format!("{} has both a proof and a disposition", line.id));
        }
    }
    for proof in proofs {
        if !known.contains(proof.capability) {
            problems.push(format!(
                "proof names unknown inventory item {}",
                proof.capability
            ));
        }
    }
    for cap in caps {
        if !listed.contains(cap.id) && !proof_ids.contains(cap.id) {
            problems.push(format!(
                "{} has no behavior example or explicit disposition",
                cap.id
            ));
        }
    }
    problems.extend(proof_problems(caps, proofs));
    problems
}

fn proof_kind_accepts(kind: ProofKind, cap: CapKind) -> bool {
    match kind {
        ProofKind::Demo => cap == CapKind::Type,
        ProofKind::Option => matches!(cap, CapKind::Variant | CapKind::Builder | CapKind::Function),
        ProofKind::StateRow => matches!(cap, CapKind::Type | CapKind::Variant),
        ProofKind::Via => matches!(cap, CapKind::Builder | CapKind::Function),
    }
}

/// Via evidence is limited to a catalogued method with a direct same-page
/// Demo for its owner. Builders need their own explicit supported contract.
fn via_owner_problems(caps: &[Capability], proofs: &[BehaviorProof]) -> Vec<String> {
    let mut problems = Vec::new();
    let by_id: HashMap<&str, &Capability> = caps.iter().map(|cap| (cap.id, cap)).collect();
    for proof in proofs.iter().filter(|proof| proof.kind == ProofKind::Via) {
        let Some(cap) = by_id.get(proof.capability) else {
            continue;
        };
        if cap.kind != CapKind::Function {
            problems.push(format!(
                "{} is not a function-method Via capability",
                proof.capability
            ));
        }
        let parent = proof.capability.rsplit_once("::").map(|(parent, _)| parent);
        if proof.owner != parent {
            problems.push(format!(
                "{} Via owner {:?} does not match method parent {:?}",
                proof.capability, proof.owner, parent
            ));
        }
        let Some(owner) = proof.owner else {
            problems.push(format!(
                "{} is a Via proof without an owner",
                proof.capability
            ));
            continue;
        };
        if !by_id
            .get(owner)
            .is_some_and(|owner_cap| owner_cap.kind == CapKind::Type)
        {
            problems.push(format!(
                "{} has no catalogued Via owner type {owner}",
                proof.capability
            ));
        }
        let has_direct_demo = proofs.iter().any(|candidate| {
            candidate.capability == owner
                && candidate.page == proof.page
                && candidate.route == proof.route
                && candidate.kind == ProofKind::Demo
                && candidate.owner.is_none()
        });
        if !has_direct_demo {
            problems.push(format!(
                "{} Via owner {owner} has no direct Demo proof on {:?}",
                proof.capability, proof.page
            ));
        }
    }
    problems
}

fn proof_problems(caps: &[Capability], proofs: &[BehaviorProof]) -> Vec<String> {
    let mut problems = Vec::new();
    let by_id: HashMap<&str, &Capability> = caps.iter().map(|cap| (cap.id, cap)).collect();

    if proofs.len() != EXECUTABLE_PROOFS.len() {
        problems.push(format!(
            "expected exactly {} supported executable proof controls, found {}",
            EXECUTABLE_PROOFS.len(),
            proofs.len()
        ));
    }
    for expected in &EXECUTABLE_PROOFS {
        if !proofs.contains(expected) {
            problems.push(format!(
                "supported executable proof control for {} is missing or modified",
                expected.capability
            ));
        }
    }

    let mut seen = HashSet::new();
    for proof in proofs {
        if !seen.insert(proof.capability) {
            problems.push(format!("behavior inventory repeats {}", proof.capability));
        }
        if !EXECUTABLE_PROOFS.contains(proof) {
            problems.push(format!(
                "{} does not match a supported executable proof contract",
                proof.capability
            ));
        }
        match by_id.get(proof.capability) {
            None => problems.push(format!(
                "proof names stale inventory item {}",
                proof.capability
            )),
            Some(cap) if !proof_kind_accepts(proof.kind, cap.kind) => problems.push(format!(
                "{} has kind {} incompatible with {:?} proof",
                proof.capability,
                cap.kind.name(),
                proof.kind
            )),
            Some(_) => {}
        }
        if proof.page.is_empty() || declarations::page_source(proof.page).is_none() {
            problems.push(format!(
                "{} has no current page source {:?}",
                proof.capability, proof.page
            ));
        }
        if !NAV_ENTRIES.iter().any(|entry| entry.label == proof.route) {
            problems.push(format!(
                "{} names unknown current navigation route {:?}",
                proof.capability, proof.route
            ));
        }
        if proof.probe.trim().is_empty() {
            problems.push(format!("{} has an empty behavior probe", proof.capability));
        }
        if proof.kind == ProofKind::Via {
            if by_id
                .get(proof.capability)
                .is_some_and(|cap| !declarations::page_uses(proof.page, proof.capability, cap.kind))
            {
                problems.push(format!(
                    "{} has no lexical Via source-use hint on {:?}",
                    proof.capability, proof.page
                ));
            }
        } else if proof.owner.is_some() {
            problems.push(format!(
                "{} has an owner on a non-Via proof",
                proof.capability
            ));
        }
    }
    problems.extend(via_owner_problems(caps, proofs));
    problems
}

/// Page-title associations are checked only for current page and ID liveness.
fn lead_problems(caps: &[Capability], leads: &[(&str, PageLead)]) -> Vec<String> {
    let mut problems = Vec::new();
    let known: HashSet<&str> = caps.iter().map(|cap| cap.id).collect();
    let mut seen = HashSet::new();
    for (id, lead) in leads {
        if !seen.insert(*id) {
            problems.push(format!("page leads repeat {id}"));
        }
        if !known.contains(id) {
            problems.push(format!("page lead names stale inventory item {id}"));
        }
        if page_by_slug(lead.page).is_none() || declarations::page_source(lead.page).is_none() {
            problems.push(format!(
                "page lead {id} names no current page source {:?}",
                lead.page
            ));
        }
        if lead.title_hint.trim().is_empty() {
            problems.push(format!("page lead {id} has an empty title hint"));
        }
    }
    problems
}

/// Fails the test with every problem.
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
    let scanned = scan::scan_library();
    assert!(
        scanned.len() > 1000,
        "the scan found too little: {}",
        scanned.len()
    );
    for item in &scanned {
        let catalogue_item = crate::coverage::catalog::find(&item.id)
            .unwrap_or_else(|| panic!("catalogue lookup missed scanned item {}", item.id));
        assert_eq!(catalogue_item.kind, item.kind, "kind drift at {}", item.at);
    }
    fail_on(
        "the catalogue drifted from the library source",
        &registry_problems(CAPABILITIES, &scanned),
    );
}

#[test]
fn every_inventory_item_has_proof_or_explicit_disposition() {
    fail_on(
        "the inventory disposition gate",
        &coverage_problems(CAPABILITIES, PROOFS, ALLOWLIST),
    );
}

#[test]
fn page_title_leads_are_current_but_non_proving() {
    fail_on(
        "page/title leads must remain current but are not behavior proof",
        &lead_problems(CAPABILITIES, PAGE_LEADS),
    );
}

#[test]
fn coverage_summary() {
    let (allow, problems) = declarations::parse_allowlist(ALLOWLIST);
    assert!(problems.is_empty(), "{problems:#?}");
    let mut per_step: BTreeMap<&str, usize> = BTreeMap::new();
    for line in &allow {
        *per_step.entry(line.step).or_default() += 1;
    }
    let mut per_page: BTreeMap<&str, usize> = BTreeMap::new();
    for (_, demo) in PAGE_LEADS {
        *per_page.entry(demo.page).or_default() += 1;
    }
    println!(
        "bounded source inventory: {} items, {} page leads, {} dispositions",
        CAPABILITIES.len(),
        PAGE_LEADS.len(),
        allow.len()
    );
    for (step, count) in &per_step {
        println!("  {step:>9}: {count}");
    }
    for (page, count) in &per_page {
        println!("  page {page:>12}: {count}");
    }
}

/// The gate must fail when a new inventory item lacks a proof or disposition.
#[test]
fn gate_requires_new_inventory_items_to_have_proof_or_disposition() {
    let mut caps = CAPABILITIES.to_vec();
    caps.push(Capability::ty("termrock-core::fixture::Gizmo"));
    caps.push(Capability::builder("termrock-core::fixture::Gizmo::glow"));
    let gaps = coverage_problems(&caps, PROOFS, ALLOWLIST);
    for id in [
        "termrock-core::fixture::Gizmo",
        "termrock-core::fixture::Gizmo::glow",
    ] {
        assert!(
            gaps.iter().any(
                |p| p.contains("no behavior example or explicit disposition") && p.contains(id)
            ),
            "{id}: {gaps:#?}"
        );
    }
    let allow = format!(
        "{ALLOWLIST}\ntermrock-core::fixture::Gizmo | fixture | SC2\n\
         termrock-core::fixture::Gizmo::glow | fixture | SC2\n"
    );
    assert_eq!(
        coverage_problems(&caps, PROOFS, &allow),
        Vec::<String>::new()
    );
}

/// The gate must fail when the catalogue drifts from the library source.
#[test]
fn gate_fails_when_the_catalogue_drifted() {
    let scanned = scan::scan_library();

    // A capability the source has and the catalogue lacks.
    let caps: Vec<Capability> = CAPABILITIES
        .iter()
        .copied()
        .filter(|cap| cap.id != "termrock-core::id::Id")
        .collect();
    let drift = registry_problems(&caps, &scanned);
    assert!(
        drift
            .iter()
            .any(|p| p.contains("missing from the catalogue")
                && p.contains("Capability::ty(\"termrock-core::id::Id\")")),
        "{drift:#?}"
    );

    // A capability the catalogue has and the source lacks.
    let mut caps = CAPABILITIES.to_vec();
    caps.push(Capability::ty("termrock-core::fixture::Gizmo"));
    let drift = registry_problems(&caps, &scanned);
    assert!(
        drift.iter().any(|p| p
            .contains("the catalogue lists termrock-core::fixture::Gizmo (type)")
            && p.contains("which the source no longer has")),
        "{drift:#?}"
    );
}

/// A stale allowlist line must fail the gate.
#[test]
fn gate_fails_on_a_stale_allowlist_line() {
    let allow = format!("{ALLOWLIST}\ntermrock-core::id::NotACapability | stale | SC2\n");
    let problems = coverage_problems(CAPABILITIES, PROOFS, &allow);
    assert!(
        problems.iter().any(|p| p.contains("stale disposition:")),
        "{problems:#?}"
    );
}

/// The allowlist syntax and disposition status must parse; the coverage gate
/// separately rejects stale IDs.
#[test]
fn allowlist_lines_are_well_formed() {
    let (allow, problems) = declarations::parse_allowlist(ALLOWLIST);
    fail_on("the allowlist is malformed", &problems);
    for line in &allow {
        assert!(!line.reason.is_empty(), "{} has no reason", line.id);
    }
    let _: Vec<Allow<'_>> = allow;
}

#[test]
fn registered_behavior_proofs_match_current_inventory_and_page_sources() {
    fail_on(
        "behavior proof metadata is stale",
        &proof_problems(CAPABILITIES, PROOFS),
    );
}

fn assert_unbound_proofs(proofs: &[BehaviorProof]) {
    let problems = proof_problems(CAPABILITIES, proofs);
    assert!(
        problems
            .iter()
            .any(|problem| problem.contains("does not match a supported executable proof contract")),
        "{problems:#?}"
    );
}

#[test]
fn proof_contract_rejects_every_mutated_binding_field_and_unknown_rows() {
    let mut capability = PROOFS.to_vec();
    capability[0].capability = PROOFS[4].capability;
    assert_unbound_proofs(&capability);

    let mut page = PROOFS.to_vec();
    page[0].page = "overview";
    assert_unbound_proofs(&page);

    let mut route = PROOFS.to_vec();
    route[0].route = "Overview";
    assert_unbound_proofs(&route);

    let mut kind = PROOFS.to_vec();
    kind[0].kind = ProofKind::StateRow;
    assert_unbound_proofs(&kind);

    let mut probe = PROOFS.to_vec();
    probe[0].probe = "config/other.toml";
    assert_unbound_proofs(&probe);

    let mut owner = PROOFS.to_vec();
    owner[0].owner = Some("termrock-editors::diff::DiffView");
    assert_unbound_proofs(&owner);

    let mut unknown = PROOFS.to_vec();
    unknown.push(BehaviorProof {
        capability: "termrock-core::id::Id",
        page: "diff",
        route: "Diff",
        kind: ProofKind::Demo,
        probe: "config/service.toml",
        owner: None,
    });
    assert_unbound_proofs(&unknown);

    let missing = &PROOFS[1..];
    let problems = proof_problems(CAPABILITIES, missing);
    assert!(
        problems
            .iter()
            .any(|problem| problem.contains("is missing or modified")),
        "{problems:#?}"
    );

    let mut duplicate = PROOFS.to_vec();
    duplicate.push(PROOFS[0]);
    assert!(
        proof_problems(CAPABILITIES, &duplicate)
            .iter()
            .any(|problem| problem.contains("behavior inventory repeats")),
        "duplicate proof row was accepted"
    );
}

#[test]
fn inventory_gate_does_not_credit_a_mutated_proof_row() {
    let mut forged = PROOFS.to_vec();
    forged[0].probe = "config/service.toml with an unexecuted extra claim";
    let problems = coverage_problems(CAPABILITIES, &forged, ALLOWLIST);
    assert!(
        problems
            .iter()
            .any(|problem| problem.contains("does not match a supported executable proof contract")),
        "{problems:#?}"
    );
}

#[test]
fn via_requires_matching_parent_and_direct_owner_demo() {
    let mut unrelated_owner = PROOFS.to_vec();
    unrelated_owner[4].owner = Some("termrock-editors::diff::DiffViewState");
    let problems = via_owner_problems(CAPABILITIES, &unrelated_owner);
    assert!(
        problems
            .iter()
            .any(|problem| problem.contains("does not match method parent")),
        "{problems:#?}"
    );

    let without_owner_demo: Vec<_> = PROOFS
        .iter()
        .copied()
        .filter(|proof| proof.capability != "termrock-editors::diff::DiffView")
        .collect();
    let problems = via_owner_problems(CAPABILITIES, &without_owner_demo);
    assert!(
        problems
            .iter()
            .any(|problem| problem.contains("has no direct Demo proof")),
        "{problems:#?}"
    );
}

#[test]
fn current_diff_route_executes_each_diff_proof_control() {
    let demo = executable_proof(ProofControl::DiffDemo);
    let draw = executable_proof(ProofControl::SharedDraw);
    assert!(
        declarations::page_source(demo.page).is_some(),
        "the Demo contract has a current source-module slug"
    );
    assert_eq!(
        NAV_ENTRIES
            .iter()
            .filter(|entry| entry.label == demo.route)
            .count(),
        1,
        "the Demo contract has one current navigation route"
    );
    let direct = diff_harness();
    let direct_text = direct.text();
    assert!(
        direct.area_of(DIFF_TABS).is_some(),
        "the Diff route mounts its page tabs"
    );
    assert!(
        direct.area_of(DIFF_VIEW).is_some(),
        "the {} control mounts the shared DiffView",
        draw.probe
    );
    assert!(
        direct_text.contains(demo.probe),
        "the {} Demo control renders the declared output\n{direct_text}",
        demo.probe
    );
    assert!(
        direct_text.contains(draw.probe),
        "the {} draw control renders the declared output\n{direct_text}",
        draw.probe
    );
    assert_control_proof(ProofControl::DiffDemo, "diff", "Diff");
    assert_control_proof(ProofControl::SharedDraw, "diff", "Diff");
    assert!(
        direct.diagnostics().is_empty(),
        "{:?}",
        direct.diagnostics()
    );

    let review_variant = executable_proof(ProofControl::ReviewVariant);
    let set_mode = executable_proof(ProofControl::SetMode);
    let update = executable_proof(ProofControl::SharedUpdate);
    let mut option = diff_harness();
    let before_change = option.text();
    assert!(
        !before_change.contains(set_mode.probe),
        "the {} control should change the initial DiffView output",
        set_mode.probe
    );
    assert!(option.tab_to(DIFF_TABS));
    let _ = option.key(KeyCode::Char('2'));
    assert!(
        option.tab_to(DIFF_MODE),
        "the current mode Select is reachable"
    );
    let _ = option.key(KeyCode::Enter);
    let _ = option.key(KeyCode::Down);
    let _ = option.key(KeyCode::Enter);
    assert!(option.tab_to(DIFF_TABS));
    let _ = option.key(KeyCode::Char('1'));
    let option_text = option.text();
    assert!(
        option_text.contains(review_variant.probe),
        "the {} control should select Review and render its marker\\n{option_text}",
        review_variant.probe
    );
    assert!(
        option_text.contains(set_mode.probe),
        "the {} control should change the rendered DiffView\\n{option_text}",
        set_mode.probe
    );
    assert!(
        option_text.contains(update.probe),
        "the {} control should update the shared component\\n{option_text}",
        update.probe
    );
    assert_control_proof(ProofControl::ReviewVariant, "diff", "Diff");
    assert_control_proof(ProofControl::SetMode, "diff", "Diff");
    assert_control_proof(ProofControl::SharedUpdate, "diff", "Diff");
    assert!(
        option.diagnostics().is_empty(),
        "{:?}",
        option.diagnostics()
    );

    let state_row = executable_proof(ProofControl::StateRow);
    let mut states = diff_harness();
    assert!(states.tab_to(DIFF_TABS));
    let _ = states.key(KeyCode::Char('3'));
    let text = states.text();
    for label in ["Unified", "Review", "Rest", "Focused", "Empty"] {
        assert!(
            text.contains(label),
            "current StateRow example lacks {label:?}\n{text}"
        );
    }
    assert!(
        text.contains(state_row.probe),
        "the {} state-row control renders the declared output\n{text}",
        state_row.probe
    );
    assert_control_proof(ProofControl::StateRow, "diff", "Diff");
    assert!(
        states.diagnostics().is_empty(),
        "{:?}",
        states.diagnostics()
    );
}

#[test]
fn current_textareas_route_executes_edit_entry_selector_behavior() {
    let proof = executable_proof(ProofControl::TextAreaEditEntry);
    let mut harness = textareas_harness();
    seed_textarea(&mut harness, "base");

    choose_textareas_option(
        &mut harness,
        TEXTAREAS_EDIT_ENTRY,
        &[KeyCode::Down],
        proof.probe,
    );
    assert!(
        harness.tab_to(TEXTAREAS_BODY),
        "the TypeToEdit body is reachable"
    );
    assert!(
        !harness
            .state_of(TEXTAREAS_BODY)
            .contains(StateFlags::EDITING)
    );
    let original = textarea_text(&harness);
    assert_eq!(original, "base", "the TypeToEdit baseline is known");

    let _ = harness.key(KeyCode::Enter);
    let _ = harness.key(KeyCode::Right);
    assert!(
        !harness
            .state_of(TEXTAREAS_BODY)
            .contains(StateFlags::EDITING)
    );
    assert_eq!(
        textarea_text(&harness),
        original,
        "idle Enter and arrows do not begin TypeToEdit"
    );

    let _ = harness.key(KeyCode::Char('x'));
    assert!(
        harness
            .state_of(TEXTAREAS_BODY)
            .contains(StateFlags::EDITING)
    );
    assert_eq!(textarea_text(&harness), "basex");
    let _ = harness.key(KeyCode::Esc);
    assert!(
        !harness
            .state_of(TEXTAREAS_BODY)
            .contains(StateFlags::EDITING)
    );
    assert_eq!(textarea_text(&harness), "basex");

    let paste = " plus\n雪";
    let _ = harness.paste(paste);
    assert!(
        harness
            .state_of(TEXTAREAS_BODY)
            .contains(StateFlags::EDITING)
    );
    let pasted = textarea_text(&harness);
    assert_eq!(
        pasted, "basex plus\n雪",
        "focused paste starts TypeToEdit and inserts the complete multiline payload"
    );
    let _ = harness.key(KeyCode::Esc);
    assert_eq!(textarea_text(&harness), "basex plus\n雪");

    choose_textareas_option(
        &mut harness,
        TEXTAREAS_EDIT_ENTRY,
        &[KeyCode::Down],
        "Explicit",
    );
    assert!(
        harness.tab_to(TEXTAREAS_BODY),
        "the Explicit body is reachable"
    );
    let explicit_before = textarea_text(&harness);
    assert!(
        !harness
            .state_of(TEXTAREAS_BODY)
            .contains(StateFlags::EDITING)
    );
    let _ = harness.key(KeyCode::Char('y'));
    assert!(
        !harness
            .state_of(TEXTAREAS_BODY)
            .contains(StateFlags::EDITING)
    );
    assert_eq!(
        textarea_text(&harness),
        explicit_before,
        "idle typing does not begin"
    );

    let _ = harness.key(KeyCode::Enter);
    assert!(
        harness
            .state_of(TEXTAREAS_BODY)
            .contains(StateFlags::EDITING)
    );
    let _ = harness.key(KeyCode::Char('y'));
    assert!(textarea_text(&harness).ends_with('y'));
    let _ = harness.key(KeyCode::Esc);
    assert!(textarea_text(&harness).ends_with('y'));

    assert_control_proof(ProofControl::TextAreaEditEntry, "textareas", "Text areas");
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn current_textareas_route_executes_focus_select_on_unicode_multiline_value() {
    let proof = executable_proof(ProofControl::TextAreaFocusSelect);
    let mut harness = textareas_harness();
    let unicode_multiline = "seed 雪\n👩‍💻";
    seed_textarea(&mut harness, unicode_multiline);
    let seeded = textarea_text(&harness);
    assert_eq!(seeded, unicode_multiline, "the Unicode seed is exact");

    choose_textareas_option(
        &mut harness,
        TEXTAREAS_FOCUS_SELECT,
        &[KeyCode::Down],
        proof.probe,
    );
    choose_textareas_option(
        &mut harness,
        TEXTAREAS_FOCUS_SELECT,
        &[KeyCode::Up],
        "Caret end",
    );
    assert!(harness.tab_to(TEXTAREAS_BODY), "CaretEnd body is reachable");
    assert!(
        harness
            .state_of(TEXTAREAS_BODY)
            .contains(StateFlags::EDITING)
    );
    let _ = harness.key(KeyCode::Char('Z'));
    let caret_end_value = textarea_text(&harness);
    assert_eq!(
        caret_end_value, "seed 雪\n👩‍💻Z",
        "CaretEnd appends after the existing Unicode lines"
    );
    let _ = harness.key(KeyCode::Esc);
    assert_eq!(textarea_text(&harness), "seed 雪\n👩‍💻Z");

    choose_textareas_option(
        &mut harness,
        TEXTAREAS_FOCUS_SELECT,
        &[KeyCode::Down],
        proof.probe,
    );
    assert!(
        harness.tab_to(TEXTAREAS_BODY),
        "SelectAll body is reachable"
    );
    assert!(
        harness
            .state_of(TEXTAREAS_BODY)
            .contains(StateFlags::EDITING)
    );
    let _ = harness.key(KeyCode::Char('X'));
    assert_eq!(
        textarea_text(&harness),
        "X",
        "SelectAll replaces both lines"
    );
    let _ = harness.key(KeyCode::Esc);
    assert_eq!(textarea_text(&harness), "X", "the replacement commits");

    assert_control_proof(ProofControl::TextAreaFocusSelect, "textareas", "Text areas");
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}

#[test]
fn current_textareas_route_executes_escape_policy_for_retained_session_origins() {
    let proof = executable_proof(ProofControl::TextAreaEscReverts);
    let mut harness = textareas_harness();
    seed_textarea(&mut harness, "base");

    choose_textareas_option(
        &mut harness,
        TEXTAREAS_BLUR_ACTION,
        &[KeyCode::Down, KeyCode::Down],
        "Keep",
    );
    choose_textareas_option(
        &mut harness,
        TEXTAREAS_EDIT_ENTRY,
        &[KeyCode::Down],
        "Type to edit",
    );
    choose_textareas_option(
        &mut harness,
        TEXTAREAS_ESCAPE_BEHAVIOR,
        &[KeyCode::Down],
        proof.probe,
    );

    assert!(
        harness.tab_to(TEXTAREAS_BODY),
        "TypeToEdit body is reachable"
    );
    let _ = harness.key(KeyCode::Char('!'));
    assert!(
        harness
            .state_of(TEXTAREAS_BODY)
            .contains(StateFlags::EDITING)
    );
    assert_eq!(textarea_text(&harness), "base!");
    choose_textareas_option(
        &mut harness,
        TEXTAREAS_EDIT_ENTRY,
        &[KeyCode::Down],
        "Explicit",
    );
    assert!(
        harness.tab_to(TEXTAREAS_BODY),
        "retained TypeToEdit session is reachable"
    );
    assert!(
        harness
            .state_of(TEXTAREAS_BODY)
            .contains(StateFlags::EDITING)
    );
    let _ = harness.key(KeyCode::Esc);
    assert!(
        !harness
            .state_of(TEXTAREAS_BODY)
            .contains(StateFlags::EDITING)
    );
    assert_eq!(textarea_text(&harness), "base", "TypeToEdit origin reverts");

    assert!(harness.tab_to(TEXTAREAS_BODY), "Explicit body is reachable");
    assert!(
        !harness
            .state_of(TEXTAREAS_BODY)
            .contains(StateFlags::EDITING)
    );
    let explicit_before = textarea_text(&harness);
    let _ = harness.key(KeyCode::Char('!'));
    assert!(
        !harness
            .state_of(TEXTAREAS_BODY)
            .contains(StateFlags::EDITING)
    );
    assert_eq!(
        textarea_text(&harness),
        explicit_before,
        "Explicit ignores idle typing"
    );
    let _ = harness.key(KeyCode::Enter);
    assert!(
        harness
            .state_of(TEXTAREAS_BODY)
            .contains(StateFlags::EDITING)
    );
    let _ = harness.key(KeyCode::Char('!'));
    assert_eq!(textarea_text(&harness), "base!");
    choose_textareas_option(
        &mut harness,
        TEXTAREAS_EDIT_ENTRY,
        &[KeyCode::Up],
        "Type to edit",
    );
    assert!(
        harness.tab_to(TEXTAREAS_BODY),
        "retained Explicit session is reachable"
    );
    assert!(
        harness
            .state_of(TEXTAREAS_BODY)
            .contains(StateFlags::EDITING)
    );
    let _ = harness.key(KeyCode::Esc);
    assert!(
        !harness
            .state_of(TEXTAREAS_BODY)
            .contains(StateFlags::EDITING)
    );
    assert_eq!(
        textarea_text(&harness),
        "base!",
        "Explicit origin commits on Escape"
    );

    assert_control_proof(ProofControl::TextAreaEscReverts, "textareas", "Text areas");
    assert!(
        harness.diagnostics().is_empty(),
        "{:?}",
        harness.diagnostics()
    );
}
