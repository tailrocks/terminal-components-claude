//! Checked fixture accounting shared by views, review and simulation effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InventoryError {
    Overflow,
    DuplicateIdentity,
    InvalidCapacity,
    InvalidTarget,
    OverlappingTargets,
}
impl std::fmt::Display for InventoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Overflow => "inventory byte total exceeds supported range",
            Self::DuplicateIdentity => "inventory target identities must be unique and nonempty",
            Self::InvalidCapacity => "inventory exceeds modeled capacity",
            Self::InvalidTarget => "invalid cleanup target",
            Self::OverlappingTargets => "cleanup targets overlap",
        })
    }
}
impl std::error::Error for InventoryError {}

pub(crate) fn bytes(values: impl IntoIterator<Item = u64>) -> Result<u64, InventoryError> {
    values.into_iter().try_fold(0_u64, |total, value| {
        total.checked_add(value).ok_or(InventoryError::Overflow)
    })
}

pub(crate) fn identities<'a>(
    values: impl IntoIterator<Item = &'a str>,
) -> Result<(), InventoryError> {
    let mut seen = std::collections::BTreeSet::new();
    for value in values {
        if value.is_empty() || !seen.insert(value) {
            return Err(InventoryError::DuplicateIdentity);
        }
    }
    Ok(())
}
