#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "integration assertions"
)]
//! Union: the components namespace joins the four application namespaces
//! without collision or duplication.

use oracle_components::{APP_NAMESPACES, NAMESPACE, expand, join_namespaces};

#[test]
fn union_joins_five_namespaces() {
    let joined = join_namespaces().expect("join");
    assert_eq!(joined.namespaces.len(), APP_NAMESPACES.len() + 1);
    for app in APP_NAMESPACES {
        assert!(joined.namespaces.contains(&app.to_owned()), "missing {app}");
    }
    assert!(joined.namespaces.contains(&NAMESPACE.to_owned()));
}

#[test]
fn union_covers_the_full_expansion_exactly_once() {
    let joined = join_namespaces().expect("join");
    let expanded = expand().expect("expand");
    assert_eq!(joined.identities.len(), expanded.len());
    let mut sorted = joined.identities.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        joined.identities.len(),
        "union has duplicates"
    );
    for identity in &joined.identities {
        assert!(identity.starts_with("components/"), "escapes: {identity}");
        for app in APP_NAMESPACES {
            assert!(
                !identity.starts_with(&format!("{app}/")),
                "collides with {app}: {identity}"
            );
        }
    }
}
