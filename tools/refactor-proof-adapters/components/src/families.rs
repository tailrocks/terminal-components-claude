//! The 54 `component-parity.tsv` family rows, classified, never selected.
//!
//! Authority remains the repository TSV, pinned here with `include_str!`.
//! [`rows`] fails closed unless the TSV holds exactly these 54 unique rows in
//! this order, so a family can neither be omitted nor invented.

use crate::error::AdapterError;

/// Pinned family inventory. Authority remains the repository TSV.
const TSV: &str = include_str!("../../../../docs/refactoring-plan/component-parity.tsv");

/// TSV header; any drift fails the parse.
const HEADER: &str = "family\treference_implementation\tmain_implementation\tarchitectural_target\tvisual_status\tinteraction_status\tapi_refactor_status\ttests_available\ttests_missing\tproposed_owner\towning_task_ids";

/// Exhaustive family ledger size.
pub const FAMILY_COUNT: usize = 54;

/// One component family from the parity ledger.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Family {
    /// Stable identity and keyed parts.
    Identity,
    /// Typed intent/response flow.
    EventResponse,
    /// Focus ring, hit publication, capture.
    FocusHitCapture,
    /// Runtime lifecycle, session clocks.
    RuntimeSessionTime,
    /// Pure layout/measure geometry.
    LayoutMeasure,
    /// Semantic recipes and paint.
    ThemePaint,
    /// Shared grapheme-safe text core.
    TextCore,
    /// Scroll model, region, fade painter.
    ScrollStateRegion,
    /// Layer stack, anchors, backdrops.
    LayersPopups,
    /// Borrowed keyed collections.
    CollectionCore,
    /// Controlled button.
    Button,
    /// Static or click-only brand lockup.
    Brand,
    /// Checkbox and toggle.
    CheckboxToggle,
    /// Keyed radio group.
    RadioGroup,
    /// Keyed chip bar.
    Chips,
    /// Field chrome wrapper.
    FieldChrome,
    /// Single-line text input.
    TextInput,
    /// Multiline text area.
    TextArea,
    /// Masked secrets and typed validation.
    SecretValidation,
    /// Dropdown select.
    Select,
    /// ID-keyed form manager.
    Form,
    /// Keyed list.
    List,
    /// Filtered list composite.
    FilterList,
    /// Navigation list composite.
    NavList,
    /// Keyed tree.
    Tree,
    /// Step tracker.
    Steps,
    /// Tab strip.
    Tabs,
    /// Command palette picker.
    PickerCommandPalette,
    /// Staged picker chain.
    PickerChain,
    /// Inline completion.
    Completion,
    /// Modal dialog.
    Dialog,
    /// Context menu, menu bar.
    MenuContextMenubar,
    /// Help overlay.
    HelpOverlay,
    /// Multi-step wizard.
    Wizard,
    /// Keyed grid.
    Grid,
    /// Read-only table over grid.
    Table,
    /// Code editor.
    CodeEditor,
    /// Adaptive diff view.
    DiffView,
    /// Retained-output text viewport.
    TextViewport,
    /// Legacy scroll panel policy.
    ScrollPanel,
    /// Container panel.
    Panel,
    /// Split pane.
    SplitPane,
    /// Property list.
    Props,
    /// Empty/readiness states.
    EmptyReadiness,
    /// Progress bar.
    ProgressBar,
    /// Spinner.
    Spinner,
    /// Capacity meter.
    Meter,
    /// Status bar and segments.
    StatusbarSegments,
    /// Hint bar.
    Hintbar,
    /// Key hint.
    Keyhint,
    /// Too-small fallback.
    TooSmall,
    /// Downstream author API surface.
    CustomAuthorApi,
    /// Test registry (architecture-only).
    TestingRegistry,
    /// Legacy compatibility removal.
    PublicCompatibilityRemoval,
}

impl Family {
    /// All 54 families in TSV order.
    pub const ALL: [Self; FAMILY_COUNT] = [
        Self::Identity,
        Self::EventResponse,
        Self::FocusHitCapture,
        Self::RuntimeSessionTime,
        Self::LayoutMeasure,
        Self::ThemePaint,
        Self::TextCore,
        Self::ScrollStateRegion,
        Self::LayersPopups,
        Self::CollectionCore,
        Self::Button,
        Self::Brand,
        Self::CheckboxToggle,
        Self::RadioGroup,
        Self::Chips,
        Self::FieldChrome,
        Self::TextInput,
        Self::TextArea,
        Self::SecretValidation,
        Self::Select,
        Self::Form,
        Self::List,
        Self::FilterList,
        Self::NavList,
        Self::Tree,
        Self::Steps,
        Self::Tabs,
        Self::PickerCommandPalette,
        Self::PickerChain,
        Self::Completion,
        Self::Dialog,
        Self::MenuContextMenubar,
        Self::HelpOverlay,
        Self::Wizard,
        Self::Grid,
        Self::Table,
        Self::CodeEditor,
        Self::DiffView,
        Self::TextViewport,
        Self::ScrollPanel,
        Self::Panel,
        Self::SplitPane,
        Self::Props,
        Self::EmptyReadiness,
        Self::ProgressBar,
        Self::Spinner,
        Self::Meter,
        Self::StatusbarSegments,
        Self::Hintbar,
        Self::Keyhint,
        Self::TooSmall,
        Self::CustomAuthorApi,
        Self::TestingRegistry,
        Self::PublicCompatibilityRemoval,
    ];

    /// TSV `family` slug; also the expansion identity segment.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Identity => "identity",
            Self::EventResponse => "event-response",
            Self::FocusHitCapture => "focus-hit-capture",
            Self::RuntimeSessionTime => "runtime-session-time",
            Self::LayoutMeasure => "layout-measure",
            Self::ThemePaint => "theme-paint",
            Self::TextCore => "text-core",
            Self::ScrollStateRegion => "scroll-state-region",
            Self::LayersPopups => "layers-popups",
            Self::CollectionCore => "collection-core",
            Self::Button => "button",
            Self::Brand => "brand",
            Self::CheckboxToggle => "checkbox-toggle",
            Self::RadioGroup => "radio-group",
            Self::Chips => "chips",
            Self::FieldChrome => "field-chrome",
            Self::TextInput => "text-input",
            Self::TextArea => "text-area",
            Self::SecretValidation => "secret-validation",
            Self::Select => "select",
            Self::Form => "form",
            Self::List => "list",
            Self::FilterList => "filter-list",
            Self::NavList => "nav-list",
            Self::Tree => "tree",
            Self::Steps => "steps",
            Self::Tabs => "tabs",
            Self::PickerCommandPalette => "picker-command-palette",
            Self::PickerChain => "picker-chain",
            Self::Completion => "completion",
            Self::Dialog => "dialog",
            Self::MenuContextMenubar => "menu-context-menubar",
            Self::HelpOverlay => "help-overlay",
            Self::Wizard => "wizard",
            Self::Grid => "grid",
            Self::Table => "table",
            Self::CodeEditor => "code-editor",
            Self::DiffView => "diff-view",
            Self::TextViewport => "text-viewport",
            Self::ScrollPanel => "scroll-panel",
            Self::Panel => "panel",
            Self::SplitPane => "split-pane",
            Self::Props => "props",
            Self::EmptyReadiness => "empty-readiness",
            Self::ProgressBar => "progress-bar",
            Self::Spinner => "spinner",
            Self::Meter => "meter",
            Self::StatusbarSegments => "statusbar-segments",
            Self::Hintbar => "hintbar",
            Self::Keyhint => "keyhint",
            Self::TooSmall => "too-small",
            Self::CustomAuthorApi => "custom-author-api",
            Self::TestingRegistry => "testing-registry",
            Self::PublicCompatibilityRemoval => "public-compatibility-removal",
        }
    }

    /// Parse a TSV slug.
    #[must_use]
    pub fn from_slug(slug: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|family| family.slug() == slug)
    }
}

/// One parsed TSV row: family plus its source-evidence columns.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FamilyRow {
    /// Classified family.
    pub family: Family,
    /// `reference_implementation` (`O:` evidence).
    pub reference_implementation: String,
    /// `main_implementation` (`M:` evidence).
    pub main_implementation: String,
    /// `architectural_target`.
    pub architectural_target: String,
    /// `owning_task_ids`.
    pub owning_task_ids: String,
}

/// Parse the pinned TSV: exactly 54 unique rows in [`Family::ALL`] order.
pub fn rows() -> Result<Vec<FamilyRow>, AdapterError> {
    let mut lines = TSV.lines();
    let Some(header) = lines.next() else {
        return Err(catalog("empty component-parity.tsv"));
    };
    if header != HEADER {
        return Err(catalog("unexpected component-parity.tsv header"));
    }
    let mut out = Vec::with_capacity(FAMILY_COUNT);
    for (index, line) in lines.enumerate() {
        if line.is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() != 11 {
            return Err(catalog(&format!(
                "line {}: expected 11 columns, got {}",
                index.saturating_add(2),
                cols.len()
            )));
        }
        let get = |i: usize| cols.get(i).copied().unwrap_or_default();
        let Some(family) = Family::from_slug(get(0)) else {
            return Err(catalog(&format!(
                "line {}: unknown family {}",
                index.saturating_add(2),
                get(0)
            )));
        };
        out.push(FamilyRow {
            family,
            reference_implementation: get(1).to_owned(),
            main_implementation: get(2).to_owned(),
            architectural_target: get(3).to_owned(),
            owning_task_ids: get(10).to_owned(),
        });
    }
    if out.len() != FAMILY_COUNT {
        return Err(catalog(&format!(
            "expected {FAMILY_COUNT} family rows, got {}",
            out.len()
        )));
    }
    for (row, expected) in out.iter().zip(Family::ALL) {
        if row.family != expected {
            return Err(catalog(&format!(
                "row order drift: TSV has {}, ledger expects {}",
                row.family.slug(),
                expected.slug()
            )));
        }
    }
    Ok(out)
}

/// Ledger rows keyed by family (same order as [`Family::ALL`]).
pub fn family_rows() -> Result<Vec<FamilyRow>, AdapterError> {
    rows()
}

fn catalog(message: &str) -> AdapterError {
    AdapterError::Catalog {
        message: message.to_owned(),
    }
}
