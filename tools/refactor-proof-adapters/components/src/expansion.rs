//! Finite expansion of the component corpus.
//!
//! Every Direct family expands across its applicable states, the CP-COMMON
//! size/origin/color axes. Scroll-fade viewports (heights 3/4/11/12, all four
//! edge positions) and retained-output mutation cases attach as facets on the
//! Base state. Identities are prefix-namespaced (`components/…`) so the union
//! with the four application namespaces cannot collide; duplicates fail closed.

use std::collections::BTreeSet;

use crate::color::{ColorSpec, Origin, TerminalSize};
use crate::disposition::{Lane, disposition};
use crate::error::AdapterError;
use crate::families::Family;
use crate::states::{ComponentState, applicable_states};
use crate::{
    ARCHITECTURE_FAMILY_COUNT, COMPOSITION_FAMILY_COUNT, DIRECT_FAMILY_COUNT, FAMILY_COUNT,
};

/// CP-01 fade proof heights: 3 has no fade, 4–11 fade one row per edge,
/// 12 and above fade two rows.
pub const FADE_HEIGHTS: [u16; 4] = [3, 4, 11, 12];

/// Fade heights carrying scroll-position proof for `family`.
///
/// The picker paints chrome only below its list threshold (h3/h4 show the
/// "Choose" frame with no list rows and no focus stop, so no scroll position
/// exists there); its position proof runs at 11/12 only. The framed scroll
/// panel leaves a single body row at h3, where short and tall content paint
/// identically; its proof runs at 4/11/12. Every height remains covered by
/// the other fade families.
#[must_use]
pub const fn fade_heights(family: Family) -> &'static [u16] {
    match family {
        Family::PickerCommandPalette => &[11, 12],
        Family::ScrollPanel => &[4, 11, 12],
        _ => &FADE_HEIGHTS,
    }
}

/// Families whose viewports carry the fade-facet expansion (CP-01 callers with
/// a scrollable viewport; plain `Panel` has no scroll offset and is excluded).
pub const FADE_FAMILIES: [Family; 12] = [
    Family::ScrollStateRegion,
    Family::List,
    Family::Tree,
    Family::Grid,
    Family::Table,
    Family::TextViewport,
    Family::TextArea,
    Family::CodeEditor,
    Family::ScrollPanel,
    Family::PickerCommandPalette,
    Family::Props,
    Family::Steps,
];

/// Scroll position inside a fade viewport.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FadePosition {
    /// Content fits; no scroll offset exists.
    Fits,
    /// Offset zero with more content below.
    Top,
    /// Offset strictly inside the content.
    Middle,
    /// Viewport ends exactly at content end.
    Bottom,
}

impl FadePosition {
    /// All four edge positions.
    pub const ALL: [Self; 4] = [Self::Fits, Self::Top, Self::Middle, Self::Bottom];

    /// Identity token.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Fits => "fits",
            Self::Top => "top",
            Self::Middle => "middle",
            Self::Bottom => "bottom",
        }
    }
}

/// Retained-output mutation cases (CP-04).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ViewportMutation {
    /// Append lines at the tail.
    Append,
    /// Replace the last line.
    ReplaceLast,
    /// Evict lines at the front.
    FrontEvict,
    /// Wholesale replacement.
    WholesaleReplace,
    /// Same-length mutation (cache-validity probe).
    SameLength,
}

impl ViewportMutation {
    /// All five mutation cases.
    pub const ALL: [Self; 5] = [
        Self::Append,
        Self::ReplaceLast,
        Self::FrontEvict,
        Self::WholesaleReplace,
        Self::SameLength,
    ];

    /// Identity token.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Append => "append",
            Self::ReplaceLast => "replace-last",
            Self::FrontEvict => "front-evict",
            Self::WholesaleReplace => "wholesale-replace",
            Self::SameLength => "same-length",
        }
    }
}

/// Extra proof facet attached to a case.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Facet {
    /// No facet: the plain state/size/origin/color frame.
    None,
    /// Fade viewport at an exact height and scroll position.
    FadeViewport {
        /// Viewport content height (3, 4, 11, or 12).
        height: u16,
        /// Scroll position.
        position: FadePosition,
    },
    /// Retained-output mutation applied before capture.
    ViewportMutation {
        /// Mutation case.
        kind: ViewportMutation,
    },
}

impl Facet {
    /// Identity token.
    #[must_use]
    pub fn token(self) -> String {
        match self {
            Self::None => "plain".to_owned(),
            Self::FadeViewport { height, position } => {
                format!("fade-h{height}-{}", position.token())
            }
            Self::ViewportMutation { kind } => format!("mut-{}", kind.token()),
        }
    }
}

/// One finite expanded capture identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExpandedCase {
    /// Component family.
    pub family: Family,
    /// Observable state.
    pub state: ComponentState,
    /// Frame size.
    pub size: TerminalSize,
    /// Widget origin inside the frame.
    pub origin: Origin,
    /// Colour capability.
    pub color: ColorSpec,
    /// Proof facet.
    pub facet: Facet,
}

impl ExpandedCase {
    /// Stable membership identity. Candidate output does not select this.
    #[must_use]
    pub fn identity(&self) -> String {
        format!(
            "components/{}/{}/{}/{}/{}/{}",
            self.family.slug(),
            self.state.token(),
            self.size.token(),
            self.origin.token(),
            self.color.token(),
            self.facet.token()
        )
    }
}

/// Expand the full finite corpus: Direct state frames plus fade and mutation
/// facets. Duplicates fail closed.
pub fn expand() -> Result<Vec<ExpandedCase>, AdapterError> {
    let mut out = Vec::new();
    for family in Family::ALL {
        if disposition(family).lane != Lane::Direct {
            continue;
        }
        for state in applicable_states(family) {
            for size in TerminalSize::AXIS {
                for origin in Origin::AXIS {
                    for color in ColorSpec::AXIS {
                        out.push(ExpandedCase {
                            family,
                            state: *state,
                            size,
                            origin,
                            color,
                            facet: Facet::None,
                        });
                    }
                }
            }
        }
        if FADE_FAMILIES.contains(&family) {
            for height in fade_heights(family).iter().copied() {
                for position in FadePosition::ALL {
                    for color in ColorSpec::AXIS {
                        out.push(ExpandedCase {
                            family,
                            state: ComponentState::Base,
                            size: TerminalSize::new(120, 40),
                            origin: Origin::AXIS[0],
                            color,
                            facet: Facet::FadeViewport { height, position },
                        });
                    }
                }
            }
        }
        if family == Family::TextViewport {
            for kind in ViewportMutation::ALL {
                for color in ColorSpec::AXIS {
                    out.push(ExpandedCase {
                        family,
                        state: ComponentState::Base,
                        size: TerminalSize::new(120, 40),
                        origin: Origin::AXIS[0],
                        color,
                        facet: Facet::ViewportMutation { kind },
                    });
                }
            }
        }
    }
    let mut seen = BTreeSet::new();
    for case in &out {
        let identity = case.identity();
        if !seen.insert(identity.clone()) {
            return Err(AdapterError::DuplicateIdentity { identity });
        }
    }
    Ok(out)
}

/// Exact inventory counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Membership {
    /// 54 parity rows parsed from the TSV.
    pub family_rows: usize,
    /// Direct families.
    pub direct_families: usize,
    /// Composition families.
    pub composition_families: usize,
    /// Architecture families.
    pub architecture_families: usize,
    /// Direct (family, state) pairs.
    pub state_pairs: usize,
    /// Plain state frames (pairs x sizes x origins x colors).
    pub plain_frames: usize,
    /// Fade facet frames.
    pub fade_frames: usize,
    /// Viewport mutation frames.
    pub mutation_frames: usize,
    /// Full finite expansion.
    pub expanded_cases: usize,
}

/// Membership facts that accounting can re-check.
pub fn membership() -> Result<Membership, AdapterError> {
    let rows = crate::families::rows()?;
    if rows.len() != FAMILY_COUNT {
        return Err(AdapterError::Catalog {
            message: format!("expected {FAMILY_COUNT} rows, got {}", rows.len()),
        });
    }
    crate::disposition::check_lane_counts()?;
    let expanded = expand()?;
    let state_pairs: usize = Family::ALL
        .into_iter()
        .filter(|family| disposition(*family).lane == Lane::Direct)
        .map(|family| applicable_states(family).len())
        .sum();
    let axis = TerminalSize::AXIS
        .len()
        .saturating_mul(Origin::AXIS.len())
        .saturating_mul(ColorSpec::AXIS.len());
    let plain_frames = state_pairs.saturating_mul(axis);
    let fade_frames: usize = FADE_FAMILIES
        .iter()
        .map(|family| {
            fade_heights(*family)
                .len()
                .saturating_mul(FadePosition::ALL.len())
                .saturating_mul(ColorSpec::AXIS.len())
        })
        .sum();
    let mutation_frames = ViewportMutation::ALL
        .len()
        .saturating_mul(ColorSpec::AXIS.len());
    let expected = plain_frames
        .saturating_add(fade_frames)
        .saturating_add(mutation_frames);
    if expanded.len() != expected {
        return Err(AdapterError::Catalog {
            message: format!(
                "expansion holds {}, formula expects {expected}",
                expanded.len()
            ),
        });
    }
    Ok(Membership {
        family_rows: rows.len(),
        direct_families: DIRECT_FAMILY_COUNT,
        composition_families: COMPOSITION_FAMILY_COUNT,
        architecture_families: ARCHITECTURE_FAMILY_COUNT,
        state_pairs,
        plain_frames,
        fade_frames,
        mutation_frames,
        expanded_cases: expanded.len(),
    })
}
