//! Opt-in regeneration for the inventory and lexical page-lead snapshots.
//!
//! The generator writes only deterministic source inventory and page-lead data.
//! It preserves the hand-reviewed proof rows and never rewrites dispositions.
//! Stale or missing disposition rows are printed for explicit review.

use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;

use showcase_ui::PageId;
use showcase_ui::coverage::{declarations, scan};

#[test]
fn regenerate_inventory_and_page_lead_proposals() {
    if std::env::var("SHOWCASE_REGENERATE_COVERAGE").as_deref() != Ok("1") {
        return;
    }
    let root = scan::workspace_root();
    let items = scan::scan_library();
    let pages: Vec<(&str, &str)> = PageId::ALL
        .iter()
        .map(|page| (page.slug(), page.title()))
        .collect();
    let mut leads = Vec::new();
    for item in &items {
        if let Some((slug, title)) = pages
            .iter()
            .find(|(slug, _)| declarations::page_uses(slug, &item.id, item.kind))
        {
            leads.push((item.id.as_str(), *slug, *title));
        }
    }
    leads.sort_by_key(|row| row.0);

    let mut catalog = String::from(
        "// Generated inventory for the syntax subset declared in scan.rs.\n// This is a drift snapshot, not a claim of complete Rust API discovery.\n\nuse super::Capability;\n\npub static CAPABILITIES: &[Capability] = &[\n",
    );
    for item in &items {
        let _ = writeln!(
            catalog,
            "    Capability::{}({:?}),",
            item.kind.ctor(),
            item.id
        );
    }
    catalog.push_str("];\n");
    write_file(
        &root,
        "crates/showcase-ui/src/coverage/catalog_data.rs",
        &catalog,
    );

    let current =
        std::fs::read_to_string(root.join("crates/showcase-ui/src/coverage/declarations_data.rs"))
            .unwrap_or_else(|error| panic!("read proof rows: {error}"));
    let Some(data) = declarations_data_source(&leads, &current) else {
        panic!("missing hand-authored proof marker");
    };
    write_file(
        &root,
        "crates/showcase-ui/src/coverage/declarations_data.rs",
        &data,
    );

    let (allow, parse_problems) = declarations::parse_allowlist(declarations::ALLOWLIST);
    assert!(
        parse_problems.is_empty(),
        "allowlist needs manual repair: {parse_problems:#?}"
    );
    let inventory: HashSet<&str> = items.iter().map(|item| item.id.as_str()).collect();
    let dispositions: HashMap<&str, _> = allow.iter().map(|line| (line.id, line)).collect();
    for id in dispositions
        .keys()
        .copied()
        .filter(|id| !inventory.contains(id))
    {
        println!("review stale disposition: {id}");
    }
    for id in inventory.iter().copied().filter(|id| {
        !dispositions.contains_key(id)
            && !declarations::PROOFS
                .iter()
                .any(|proof| proof.capability == *id)
    }) {
        println!(
            "proposed disposition: {id} | current inventory item; behavior evidence pending review | review"
        );
    }
    println!("updated catalog_data.rs and PAGE_LEADS; allowlist dispositions were not written");
}

const PROOF_MARKER: &str = "/// Small current-head examples for distinct declaration forms.";

/// Builds regenerated declarations without dropping imports used by preserved
/// hand-authored proof rows.
fn declarations_data_source(leads: &[(&str, &str, &str)], current: &str) -> Option<String> {
    let start = current.find(PROOF_MARKER)?;
    let mut data = String::from(
        "// Lexical page/title associations are leads only, not behavior proofs.\n\nuse super::{BehaviorProof, PageLead, ProofKind};\n\npub static PAGE_LEADS: &[(&str, PageLead)] = &[\n",
    );
    for (id, page, title) in leads {
        let _ = writeln!(
            data,
            "    ({id:?}, PageLead {{ page: {page:?}, title_hint: {title:?} }}),"
        );
    }
    data.push_str("];\n\n");
    data.push_str(&current[start..]);
    Some(data)
}

#[test]
fn generated_declarations_keep_preserved_proof_names_in_scope() {
    let current = concat!(
        "/// Small current-head examples for distinct declaration forms.\n",
        "pub static PROOFS: &[BehaviorProof] = &[BehaviorProof { kind: ProofKind::Demo }];\n",
    );
    let generated = declarations_data_source(&[], current);
    assert!(
        generated.as_ref().is_some_and(|source| {
            source.contains("use super::{BehaviorProof, PageLead, ProofKind};")
                && source.contains("pub static PAGE_LEADS: &[(&str, PageLead)]")
                && source.contains("BehaviorProof { kind: ProofKind::Demo }")
        }),
        "the regenerated module must import names used by its retained proof suffix"
    );
}

fn write_file(root: &std::path::Path, relative: &str, text: &str) {
    let path = root.join(relative);
    std::fs::write(&path, text).unwrap_or_else(|error| panic!("write {}: {error}", path.display()));
}
