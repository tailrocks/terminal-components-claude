//! Catalog construction/discovery projections, manifests, ranking and usage learning rules.
#![forbid(unsafe_code)]

pub use holla_domain::manifest;
pub use holla_domain::ranking::*;
pub use holla_domain::usage::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_usage_constants() {
        assert_eq!(MAX_USES, 20);
        let half_life = HALF_LIFE_DAYS;
        assert_eq!(half_life, 10.0);
    }
}
