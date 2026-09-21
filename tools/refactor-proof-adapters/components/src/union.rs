//! Namespace union: the four accepted application namespaces plus `components`.
//!
//! Every expansion identity carries its namespace prefix, so the complete
//! oracle union is disjoint by construction. Actual receipt resolution (hash
//! pins, accepted bundle binding) is verifier-owned through the per-check
//! contexts; this module proves the adapter side cannot collide or duplicate.

use std::collections::BTreeSet;

use crate::NAMESPACE;
use crate::error::AdapterError;
use crate::expansion::expand;

/// Accepted application namespaces joined with [`NAMESPACE`].
pub const APP_NAMESPACES: [&str; 4] = ["showcase", "holla", "jackin", "tablepro"];

/// Union membership: every components identity plus the joined namespace set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NamespaceJoin {
    /// Joined namespaces in fixed order.
    pub namespaces: Vec<String>,
    /// Components identities in the union.
    pub identities: Vec<String>,
}

/// Join the application namespaces with the components expansion.
///
/// Fails closed on an empty expansion, a missing namespace prefix, or a
/// duplicate identity within the union.
pub fn join_namespaces() -> Result<NamespaceJoin, AdapterError> {
    let cases = expand()?;
    if cases.is_empty() {
        return Err(AdapterError::Catalog {
            message: "components expansion is empty; nothing to join".to_owned(),
        });
    }
    let prefix = format!("{NAMESPACE}/");
    let mut identities = Vec::with_capacity(cases.len());
    let mut seen = BTreeSet::new();
    for case in &cases {
        let identity = case.identity();
        if !identity.starts_with(&prefix) {
            return Err(AdapterError::Catalog {
                message: format!("identity {identity} escapes the components namespace"),
            });
        }
        for app in APP_NAMESPACES {
            let app_prefix = format!("{app}/");
            if identity.starts_with(&app_prefix) {
                return Err(AdapterError::Catalog {
                    message: format!("identity {identity} collides with {app}"),
                });
            }
        }
        if !seen.insert(identity.clone()) {
            return Err(AdapterError::DuplicateIdentity { identity });
        }
        identities.push(identity);
    }
    let mut namespaces: Vec<String> = APP_NAMESPACES.iter().map(ToString::to_string).collect();
    namespaces.push(NAMESPACE.to_owned());
    Ok(NamespaceJoin {
        namespaces,
        identities,
    })
}
