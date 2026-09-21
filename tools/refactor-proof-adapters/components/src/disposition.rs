//! Per-family observation lane and old-to-new semantic mapping.
//!
//! Every one of the 54 families carries exactly one [`Lane`]. Direct families
//! render through a production widget fixture; Composition families are
//! observed through shared corpus frames plus an explicit mapping; Architecture
//! families have an explicit non-frame disposition bound to a future owner.
//! A frame request for an Architecture family is [`AdapterError::ArchitectureHasNoFrame`].

use crate::error::AdapterError;
use crate::families::Family;
use crate::{ARCHITECTURE_FAMILY_COUNT, COMPOSITION_FAMILY_COUNT, DIRECT_FAMILY_COUNT};

/// How a family is observed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Lane {
    /// Renders directly through a production widget/view fixture.
    Direct,
    /// Observed through shared corpus compositions; no own frame.
    Composition,
    /// Architecture-only; an explicit non-frame disposition.
    Architecture,
}

/// Non-frame disposition details.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Disposition {
    /// Observation lane.
    pub lane: Lane,
    /// Source-backed rationale (parity columns, CP item, or contract clause).
    pub rationale: &'static str,
}

/// Old-to-new semantic mapping for renamed, absorbed, or new-architecture
/// components. New-architecture components are proven through matching oracle
/// compositions; no old screenshot is invented for them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OldNewMapping {
    /// Oracle-side composition or module the new component must reproduce.
    pub oracle_side: &'static str,
    /// Production component that owns the rendering.
    pub production_side: &'static str,
    /// What the architecture fixture shows.
    pub fixture: &'static str,
}

/// Observation lane plus rationale for every family.
#[must_use]
#[allow(
    clippy::too_many_lines,
    reason = "one rationale arm per family by design"
)]
pub const fn disposition(family: Family) -> Disposition {
    match family {
        Family::Identity => Disposition {
            lane: Lane::Architecture,
            rationale: "keyed identity is API/behavioral: stable semantic identity under reorder/insert/remove has no standalone oracle widget frame",
        },
        Family::EventResponse => Disposition {
            lane: Lane::Architecture,
            rationale: "intent/response flow is behavioral: consume/change distinctions and action counts per event sequence, not a widget frame",
        },
        Family::FocusHitCapture => Disposition {
            lane: Lane::Architecture,
            rationale: "CP-02/CP-07 are event-sequence proofs (Down-only, release-outside, disabled barriers); focus/hover/pressed paint is observed inside every Direct fixture frame",
        },
        Family::RuntimeSessionTime => Disposition {
            lane: Lane::Architecture,
            rationale: "PTY lifecycle, suspend/resume, cancellation, and flood fairness are lifecycle proofs owned by the settled-transition suite, not a widget frame",
        },
        Family::LayoutMeasure => Disposition {
            lane: Lane::Composition,
            rationale: "pure layout/measure is observed through every Direct frame's geometry plus the explicit zero/nonzero origin axis and 120x40/40x10 allocations",
        },
        Family::ThemePaint => Disposition {
            lane: Lane::Composition,
            rationale: "recipes and capability palettes are observed through the full Direct corpus across all four color levels; Paper/API cases stay architecture-typed and distinct from oracle-Junie frames",
        },
        Family::TextCore => Disposition {
            lane: Lane::Composition,
            rationale: "shared grapheme-safe text is observed through the text-input/text-area/text-viewport/code-editor/diff-view Direct frames (Unicode, selection, cursor, tabs, controls)",
        },
        Family::ScrollStateRegion => Disposition {
            lane: Lane::Direct,
            rationale: "O:scrollbar/ui-fade renders through the production ScrollRegion fixture; fade viewports observe current production paint without faking the missing CP-01 fade",
        },
        Family::LayersPopups => Disposition {
            lane: Lane::Composition,
            rationale: "layer stack, anchors, and backdrops are observed through the dialog/menu/completion/picker/help/wizard Direct overlay frames and their recorded hit layers",
        },
        Family::CollectionCore => Disposition {
            lane: Lane::Composition,
            rationale: "borrowed keyed sources and reconciliation are observed through the list/tree/grid/nav-list/steps/chips Direct frames with stable ItemKey identities",
        },
        Family::Button => Disposition {
            lane: Lane::Direct,
            rationale: "O:widgets/button renders through the production Button fixture",
        },
        Family::Brand => Disposition {
            lane: Lane::Direct,
            rationale: "O:widgets/brand renders through the production Brand fixture",
        },
        Family::CheckboxToggle => Disposition {
            lane: Lane::Direct,
            rationale: "O:widgets/choice renders through the production Checkbox/Toggle fixture",
        },
        Family::RadioGroup => Disposition {
            lane: Lane::Direct,
            rationale: "O:widgets/choice radio renders through the production RadioGroup fixture",
        },
        Family::Chips => Disposition {
            lane: Lane::Direct,
            rationale: "O:widgets/chips renders through the production ChipBar fixture (rename mapping)",
        },
        Family::FieldChrome => Disposition {
            lane: Lane::Direct,
            rationale: "O:field chrome renders through the production Field fixture",
        },
        Family::TextInput => Disposition {
            lane: Lane::Direct,
            rationale: "O:widgets/input renders through the production TextInput fixture",
        },
        Family::TextArea => Disposition {
            lane: Lane::Direct,
            rationale: "O:widgets/textarea renders through the production TextArea fixture",
        },
        Family::SecretValidation => Disposition {
            lane: Lane::Direct,
            rationale: "oracle masked fields and validators render through the production secret TextInput/Form fixture",
        },
        Family::Select => Disposition {
            lane: Lane::Direct,
            rationale: "O:widgets/select renders through the production Select fixture",
        },
        Family::Form => Disposition {
            lane: Lane::Direct,
            rationale: "new-architecture Form reproduces the oracle connection/prompt form compositions (explicit mapping, no invented old frame)",
        },
        Family::List => Disposition {
            lane: Lane::Direct,
            rationale: "O:widgets/list renders through the production List fixture",
        },
        Family::FilterList => Disposition {
            lane: Lane::Direct,
            rationale: "new-architecture FilterList reproduces the oracle finder/filter compositions (explicit mapping)",
        },
        Family::NavList => Disposition {
            lane: Lane::Direct,
            rationale: "new-architecture NavList reproduces the oracle navigation-list compositions (explicit mapping)",
        },
        Family::Tree => Disposition {
            lane: Lane::Direct,
            rationale: "O:widgets/tree renders through the production Tree fixture",
        },
        Family::Steps => Disposition {
            lane: Lane::Direct,
            rationale: "O:widgets/steps renders through the production Steps fixture",
        },
        Family::Tabs => Disposition {
            lane: Lane::Direct,
            rationale: "O:widgets/tabs renders through the production Tabs fixture",
        },
        Family::PickerCommandPalette => Disposition {
            lane: Lane::Direct,
            rationale: "O:widgets/picker renders through the production Picker/CommandPalette fixture",
        },
        Family::PickerChain => Disposition {
            lane: Lane::Direct,
            rationale: "new-architecture PickerChain reproduces the oracle staged-picker compositions (explicit mapping)",
        },
        Family::Completion => Disposition {
            lane: Lane::Direct,
            rationale: "O:widgets/completion renders through the production Completion fixture",
        },
        Family::Dialog => Disposition {
            lane: Lane::Direct,
            rationale: "O:widgets/dialog renders through the production Dialog fixture",
        },
        Family::MenuContextMenubar => Disposition {
            lane: Lane::Direct,
            rationale: "O:widgets/menu renders through the production Menu/MenuBar/ContextMenu fixture",
        },
        Family::HelpOverlay => Disposition {
            lane: Lane::Direct,
            rationale: "new-architecture HelpOverlay reproduces the oracle help compositions (explicit mapping)",
        },
        Family::Wizard => Disposition {
            lane: Lane::Direct,
            rationale: "new-architecture Wizard reproduces the oracle multi-step flow compositions (explicit mapping)",
        },
        Family::Grid => Disposition {
            lane: Lane::Direct,
            rationale: "O:widgets/grid renders through the production Grid fixture",
        },
        Family::Table => Disposition {
            lane: Lane::Direct,
            rationale: "O:widgets/table converges on the production Grid fixture in read-only table policy (explicit mapping)",
        },
        Family::CodeEditor => Disposition {
            lane: Lane::Direct,
            rationale: "O:widgets/code renders through the production CodeEditor fixture",
        },
        Family::DiffView => Disposition {
            lane: Lane::Direct,
            rationale: "O:widgets/diff renders through the production DiffView fixture",
        },
        Family::TextViewport => Disposition {
            lane: Lane::Direct,
            rationale: "O:widgets/viewport renders through the production TextViewport fixture",
        },
        Family::ScrollPanel => Disposition {
            lane: Lane::Direct,
            rationale: "legacy O:ScrollPanel converges on a production Panel plus TextViewport composition fixture (explicit mapping)",
        },
        Family::Panel => Disposition {
            lane: Lane::Direct,
            rationale: "O:widgets/panel renders through the production Panel fixture",
        },
        Family::SplitPane => Disposition {
            lane: Lane::Direct,
            rationale: "O:widgets/splitter renders through the production SplitPane fixture (rename mapping)",
        },
        Family::Props => Disposition {
            lane: Lane::Direct,
            rationale: "O:widgets/props renders through the production Props fixture",
        },
        Family::EmptyReadiness => Disposition {
            lane: Lane::Direct,
            rationale: "O:widgets/empty renders through the production Empty fixture across readiness states",
        },
        Family::ProgressBar => Disposition {
            lane: Lane::Direct,
            rationale: "O:widgets/progress bar renders through the production ProgressBar fixture",
        },
        Family::Spinner => Disposition {
            lane: Lane::Direct,
            rationale: "O:widgets/progress spinner renders through the production Spinner fixture",
        },
        Family::Meter => Disposition {
            lane: Lane::Direct,
            rationale: "new-architecture Meter reproduces the oracle capacity-meter compositions (explicit mapping)",
        },
        Family::StatusbarSegments => Disposition {
            lane: Lane::Direct,
            rationale: "O:segments/statusbar converge on the production StatusBar fixture (explicit mapping)",
        },
        Family::Hintbar => Disposition {
            lane: Lane::Direct,
            rationale: "O:widgets/hintbar renders through the production HintBar fixture",
        },
        Family::Keyhint => Disposition {
            lane: Lane::Direct,
            rationale: "O:widgets/keyhint renders through the production KeyHint fixture",
        },
        Family::TooSmall => Disposition {
            lane: Lane::Direct,
            rationale: "new-architecture TooSmall reproduces the oracle narrow-allocation fallback compositions (explicit mapping)",
        },
        Family::CustomAuthorApi => Disposition {
            lane: Lane::Architecture,
            rationale: "downstream author examples and the Segmented teaching obligation are API checks (props construction path, forbidden legacy paths), not oracle frames",
        },
        Family::TestingRegistry => Disposition {
            lane: Lane::Architecture,
            rationale: "testing-registry is architecture-only, not an oracle widget: explicit non-frame disposition; attribution/conformance obligations bind TASK-073 and TASK-031; no screenshot, direct-capture, or PTY identity exists",
        },
        Family::PublicCompatibilityRemoval => Disposition {
            lane: Lane::Architecture,
            rationale: "legacy-path removal and export-census checks are API absence proofs, not oracle frames",
        },
    }
}

/// Future owner tasks bound by an Architecture disposition.
#[must_use]
pub const fn architecture_owners(family: Family) -> &'static [&'static str] {
    match family {
        Family::Identity => &["TASK-015"],
        Family::EventResponse | Family::RuntimeSessionTime => &["TASK-009"],
        Family::FocusHitCapture => &["TASK-010"],
        Family::CustomAuthorApi | Family::PublicCompatibilityRemoval => &["TASK-031"],
        Family::TestingRegistry => &["TASK-073", "TASK-031"],
        _ => &[],
    }
}

/// Explicit old-to-new semantic mapping, when the production component is
/// renamed, absorbed, converged, or introduced by the new architecture.
#[must_use]
pub const fn mapping(family: Family) -> Option<OldNewMapping> {
    match family {
        Family::ScrollStateRegion => Some(OldNewMapping {
            oracle_side: "O:widgets/scrollbar plus O:ui/fade applied by content callers",
            production_side: "M:components/scroll_region ScrollRegion plus shared fade painter",
            fixture: "ScrollRegion geometry/boundary fixture; fade viewports record current production cells",
        }),
        Family::Chips => Some(OldNewMapping {
            oracle_side: "O:widgets/chips",
            production_side: "M:components/chip ChipBar (rename)",
            fixture: "ChipBar fixture with keyed checked/overflow states",
        }),
        Family::Form => Some(OldNewMapping {
            oracle_side: "oracle connection/prompt/dialog form compositions",
            production_side: "M:components/form Form (new architecture)",
            fixture: "Form fixture managing field states by stable Id",
        }),
        Family::FilterList => Some(OldNewMapping {
            oracle_side: "oracle finder and filtered-list compositions",
            production_side: "M:components/filter_list FilterList (new architecture)",
            fixture: "FilterList fixture with query plus keyed rows",
        }),
        Family::NavList => Some(OldNewMapping {
            oracle_side: "oracle navigation-list compositions",
            production_side: "M:components/nav_list NavList (new architecture)",
            fixture: "NavList fixture with mode plus keyed rows",
        }),
        Family::PickerChain => Some(OldNewMapping {
            oracle_side: "oracle staged-picker compositions",
            production_side: "M:components/picker_chain PickerChain (new architecture)",
            fixture: "PickerChain fixture across its stages",
        }),
        Family::HelpOverlay => Some(OldNewMapping {
            oracle_side: "oracle help compositions",
            production_side: "M:components/help HelpOverlay (new architecture)",
            fixture: "HelpOverlay fixture with help sections",
        }),
        Family::Wizard => Some(OldNewMapping {
            oracle_side: "oracle multi-step flow compositions",
            production_side: "M:components/wizard Wizard (new architecture)",
            fixture: "Wizard fixture across its steps",
        }),
        Family::Table => Some(OldNewMapping {
            oracle_side: "O:widgets/table DataTable",
            production_side: "M:components/grid Grid in read-only table policy (converged)",
            fixture: "Grid fixture with row navigation and read-only activation",
        }),
        Family::ScrollPanel => Some(OldNewMapping {
            oracle_side: "O:ScrollPanel (tailing log and non-tailing prose policies)",
            production_side: "M:Panel plus TextViewport composition (converged)",
            fixture: "Panel plus TextViewport fixture with explicit follow policy",
        }),
        Family::SplitPane => Some(OldNewMapping {
            oracle_side: "O:widgets/splitter",
            production_side: "M:components/split SplitPane (rename)",
            fixture: "SplitPane fixture with seam geometry",
        }),
        Family::Meter => Some(OldNewMapping {
            oracle_side: "oracle capacity-meter compositions",
            production_side: "M:components/meter Meter (new architecture)",
            fixture: "Meter fixture with tone and visual states",
        }),
        Family::StatusbarSegments => Some(OldNewMapping {
            oracle_side: "O:segments plus O:statusbar",
            production_side: "M:components/status StatusBar (absorbed)",
            fixture: "StatusBar fixture with status items",
        }),
        Family::TooSmall => Some(OldNewMapping {
            oracle_side: "oracle narrow-allocation fallback compositions",
            production_side: "M:components/too_small TooSmall (new architecture)",
            fixture: "TooSmall fixture at narrow allocations",
        }),
        _ => None,
    }
}

/// Assert the 42/5/7 lane split.
pub fn check_lane_counts() -> Result<(), AdapterError> {
    let (mut direct, mut composition, mut architecture) = (0_usize, 0_usize, 0_usize);
    for family in Family::ALL {
        match disposition(family).lane {
            Lane::Direct => direct = direct.saturating_add(1),
            Lane::Composition => composition = composition.saturating_add(1),
            Lane::Architecture => architecture = architecture.saturating_add(1),
        }
    }
    if direct != DIRECT_FAMILY_COUNT
        || composition != COMPOSITION_FAMILY_COUNT
        || architecture != ARCHITECTURE_FAMILY_COUNT
    {
        return Err(AdapterError::Catalog {
            message: format!(
                "lane split is {direct}/{composition}/{architecture}, expected {DIRECT_FAMILY_COUNT}/{COMPOSITION_FAMILY_COUNT}/{ARCHITECTURE_FAMILY_COUNT}"
            ),
        });
    }
    Ok(())
}
